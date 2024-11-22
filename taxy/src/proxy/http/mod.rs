use self::{error::ProxyError, pool::ConnectionPool, route::Router};
use super::{tls::TlsTermination, PortContextEvent};
use crate::server::cert_list::CertList;
use arc_swap::{ArcSwap, Cache};
use http_body_util::{combinators::BoxBody, BodyExt};
use hyper::{
    body::Incoming,
    header::{HOST, LOCATION},
    http::{
        uri::{Parts, Scheme},
        HeaderValue,
    },
    service::service_fn,
    Request, Response, StatusCode, Uri,
};
use hyper_util::{
    rt::{TokioExecutor, TokioIo},
    server::conn::auto,
};
use rewriter::{RequestRewriter, ResponseRewriter};
use std::{net::SocketAddr, str::FromStr, sync::Arc, time::SystemTime};
use taxy_api::error::Error;
use taxy_api::port::{PortStatus, SocketState};
use taxy_api::{port::PortEntry, proxy::ProxyEntry};
use tokio::{
    io::{AsyncRead, AsyncWrite, BufStream},
    sync::Notify,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use tokio_rustls::{
    rustls::{pki_types::ServerName, ClientConfig, RootCertStore},
    TlsAcceptor,
};
use tracing::{debug, error, info, span, Instrument, Level, Span};

mod error;
mod filter;
mod hyper_tls;
mod pool;
mod rewriter;
mod route;

const MAX_BUFFER_SIZE: usize = 4096;
const HTTP2_MAX_FRAME_SIZE: usize = 16384;

#[derive(Debug)]
pub struct HttpPortContext {
    pub listen: SocketAddr,
    status: PortStatus,
    span: Span,
    tls_termination: Option<TlsTermination>,
    tls_client_config: Arc<ClientConfig>,
    shared: Arc<ArcSwap<SharedContext>>,
    stop_notifier: Arc<Notify>,
}

impl HttpPortContext {
    pub fn new(entry: &PortEntry) -> Result<Self, Error> {
        let span = span!(
            Level::INFO,
            "proxy",
            resource_id = entry.id.to_string(),
            listen = %entry.port.listen
        );
        let enter = span.clone();
        let _enter = enter.enter();

        info!("initializing http proxy");
        let listen = entry.port.listen.socket_addr()?;

        let tls_termination = if let Some(tls) = &entry.port.opts.tls_termination {
            let alpn = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
            Some(TlsTermination::new(tls, alpn)?)
        } else if entry.port.listen.is_tls() {
            return Err(Error::TlsTerminationConfigMissing);
        } else {
            None
        };

        Ok(Self {
            listen,
            status: Default::default(),
            span,
            tls_termination,
            tls_client_config: Arc::new(
                ClientConfig::builder()
                    .with_root_certificates(RootCertStore::empty())
                    .with_no_client_auth(),
            ),
            shared: Arc::new(ArcSwap::from_pointee(SharedContext {
                router: Default::default(),
                header_rewriter: Default::default(),
            })),
            stop_notifier: Arc::new(Notify::new()),
        })
    }

    pub async fn setup(
        &mut self,
        ports: &[PortEntry],
        certs: &CertList,
        proxies: Vec<ProxyEntry>,
    ) -> Result<(), Error> {
        let https_ports = ports
            .iter()
            .filter(|entry| entry.port.listen.is_http() && entry.port.listen.is_tls())
            .filter(|entry| {
                proxies
                    .iter()
                    .any(|proxy| proxy.proxy.ports.contains(&entry.id))
            })
            .collect::<Vec<_>>();
        let https_port = if self.listen.is_ipv4() {
            https_ports.iter().find(|entry| {
                entry
                    .port
                    .listen
                    .ip_addr()
                    .map(|ip| ip.is_ipv4())
                    .unwrap_or_default()
            })
        } else {
            https_ports.iter().find(|entry| {
                entry
                    .port
                    .listen
                    .ip_addr()
                    .map(|ip| ip.is_ipv6())
                    .unwrap_or_default()
            })
        }
        .or(https_ports.first())
        .and_then(|entry| entry.port.listen.port().ok());

        self.shared.store(Arc::new(SharedContext {
            router: Router::new(proxies, https_port),
            header_rewriter: RequestRewriter::builder()
                .trust_upstream_headers(false)
                .set_via(HeaderValue::from_static("taxy"))
                .build(),
        }));

        let config = ClientConfig::builder()
            .with_root_certificates(certs.root_certs().clone())
            .with_no_client_auth();
        self.tls_client_config = Arc::new(config);

        if let Some(tls) = &mut self.tls_termination {
            self.status.state.tls = Some(tls.setup(certs).await);
        }
        Ok(())
    }

    pub fn apply(&mut self, new: Self) {
        *self = Self {
            stop_notifier: self.stop_notifier.clone(),
            ..new
        };
    }

    pub fn event(&mut self, event: PortContextEvent) {
        match event {
            PortContextEvent::SocketStateUpdated(state) => {
                if self.status.state.socket != state {
                    self.status.started_at = if state == SocketState::Listening {
                        Some(SystemTime::now())
                    } else {
                        None
                    };
                }
                self.status.state.socket = state;
            }
        }
    }

    pub fn status(&self) -> &PortStatus {
        &self.status
    }

    pub fn reset(&mut self) {
        self.stop_notifier.notify_waiters();
    }

    pub fn start_proxy(&mut self, stream: BufStream<TcpStream>) {
        let span = self.span.clone();

        let tls_client_config = self.tls_client_config.clone();
        let tls_acceptor = self
            .tls_termination
            .as_ref()
            .and_then(|tls| tls.acceptor.clone());

        let stop_notifier = self.stop_notifier.clone();
        let shared_cache = Cache::new(Arc::clone(&self.shared));
        let span_cloned = span.clone();

        tokio::spawn(
            async move {
                if let Err(err) = start(
                    stream,
                    tls_client_config,
                    tls_acceptor,
                    shared_cache,
                    stop_notifier,
                    span_cloned,
                )
                .await
                {
                    error!("{err}");
                }
            }
            .instrument(span),
        );
    }
}

async fn start(
    mut stream: BufStream<TcpStream>,
    tls_client_config: Arc<ClientConfig>,
    tls_acceptor: Option<TlsAcceptor>,
    shared_cache: Cache<Arc<ArcSwap<SharedContext>>, Arc<SharedContext>>,
    stop_notifier: Arc<Notify>,
    span: Span,
) -> anyhow::Result<()> {
    let local = stream.get_ref().local_addr()?;
    let remote = stream.get_ref().peer_addr()?;
    let (mut client_stream, server_stream) = tokio::io::duplex(MAX_BUFFER_SIZE);

    let first_byte = stream.read_u8().await?;
    client_stream.write_u8(first_byte).await?;

    tokio::spawn(
        async move {
            tokio::select! {
                result = tokio::io::copy_bidirectional(&mut stream, &mut client_stream) => {
                    if let Err(err) = result {
                        error!("{err}");
                    }
                },
                _ = stop_notifier.notified() => {
                    debug!("stop");
                },
            }
        }
        .instrument(span.clone()),
    );

    if tls_acceptor.is_some() && local.port() != 80 && first_byte != 0x16 {
        tokio::task::spawn(
            async move {
                let server_stream = TokioIo::new(server_stream);
                if let Err(err) = auto::Builder::new(TokioExecutor::new())
                    .serve_connection(server_stream, service_fn(redirect))
                    .await
                {
                    error!("Failed to serve the connection: {:?}", err);
                }
            }
            .instrument(span.clone()),
        );
        return Ok(());
    }

    let mut stream: Box<dyn IoStream> = Box::new(server_stream);
    let mut server_http2 = false;
    let mut sni = None;

    let forwarded_proto = if tls_acceptor.is_some() {
        "https"
    } else {
        "http"
    };

    if let Some(acceptor) = tls_acceptor {
        debug!(%remote, "server: tls handshake");
        let accepted = acceptor.accept(stream).await?;
        let tls_conn = &accepted.get_ref().1;
        server_http2 = tls_conn.alpn_protocol() == Some(b"h2");
        sni = tls_conn.server_name().map(|sni| sni.to_string());
        stream = Box::new(accepted);
    }

    let pool = Arc::new(ConnectionPool::new(tls_client_config));
    let span_cloned = span.clone();
    let service = hyper::service::service_fn(move |mut req: Request<Incoming>| {
        let mut shared_cache = shared_cache.clone();
        let span = span_cloned.clone();
        let enter = span.clone();
        let _enter = enter.enter();

        let header_host = req
            .headers()
            .get(HOST)
            .and_then(|h| h.to_str().ok().and_then(|host| host.split(':').next()));

        let domain_fronting = match (&sni, header_host) {
            (Some(sni), Some(header)) => !sni.eq_ignore_ascii_case(header),
            _ => false,
        };

        let host = header_host
            .or(sni.as_deref())
            .or(req.uri().host())
            .or(header_host);

        let header_host = header_host.map(|h| h.to_string());
        let action = format!("{} {}", req.method().as_str(), req.uri());
        let pool = pool.clone();
        let shared = shared_cache.load();

        let mut response_rewriter = ResponseRewriter::builder();
        let req = if domain_fronting {
            ProxiedRequest::Err(ProxyError::DomainFrontingDetected)
        } else if let Some((parsed, res, route)) = shared.router.get_route(&req, host) {
            let resource_id = route.resource_id;

            let mut redirect = None;
            response_rewriter = response_rewriter.https_port(route.https_port);
            if forwarded_proto == "http" && route.upgrade_insecure {
                if let Some(port) = route.https_port {
                    if let Some(uri) = header_host
                        .as_ref()
                        .and_then(|host| host.parse::<Uri>().ok())
                    {
                        let mut parts = Parts::from(uri);
                        parts.scheme = Some(Scheme::HTTPS);
                        if let Some(authority) = parts.authority {
                            parts.authority = format!("{}:{}", authority.host(), port).parse().ok();
                        }
                        parts.path_and_query = req.uri().path_and_query().cloned();
                        if let Ok(uri) = Uri::from_parts(parts) {
                            redirect = Response::builder()
                                .status(301)
                                .header(LOCATION, uri.to_string())
                                .body(String::new())
                                .ok();
                        }
                    }
                }
            }

            if let Some(redirect) = redirect {
                ProxiedRequest::Redirect(redirect)
            } else {
                if let Some(server) = parsed.servers.first() {
                    let mut url = server.url.0.clone();
                    if let Ok(mut segments) = url.path_segments_mut() {
                        segments.extend(res.path_segments);
                    }
                    url.set_query(req.uri().query());
                    if let Ok(uri) = Uri::from_str(url.as_str()) {
                        *req.uri_mut() = uri;
                    }
                }

                info!(target: "taxy::access_log", remote = %remote, %local, action, target = %req.uri());
                let span: Span = span!(Level::INFO, "http", %resource_id, remote = %remote, %local, action, target = %req.uri());

                if let Some(host) = req
                    .uri()
                    .authority()
                    .and_then(|host| HeaderValue::from_str(host.as_str()).ok())
                {
                    req.headers_mut().insert(HOST, host);
                }

                shared.header_rewriter.pre_process(
                    req.headers_mut(),
                    remote.ip(),
                    header_host.map(|h| h.to_string()),
                    forwarded_proto,
                );
                shared.header_rewriter.post_process(req.headers_mut());
                ProxiedRequest::Ok(req, span)
            }
        } else {
            ProxiedRequest::Err(ProxyError::NoRouteFound)
        };

        async move {
            response_rewriter.build().map_response(match req {
                ProxiedRequest::Ok(req, span) => {
                    pool.request(req.map(|b| BoxBody::new(b.map_err(Into::into))))
                        .instrument(span)
                        .await
                }
                ProxiedRequest::Redirect(resp) => {
                    Ok(resp.map(|b| BoxBody::new(b.map_err(Into::into))))
                }
                ProxiedRequest::Err(err) => Err(err.into()),
            })
        }
        .instrument(span)
    });

    tokio::task::spawn(
        async move {
            let stream = TokioIo::new(stream);
            let builder = auto::Builder::new(TokioExecutor::new());
            let builder = if server_http2 {
                builder.http2_only()
            } else {
                builder
            };
            let http = builder.serve_connection_with_upgrades(stream, service);
            if let Err(err) = http.await {
                error!("Failed to serve the connection: {:?}", err);
            }
        }
        .instrument(span.clone()),
    );

    Ok(())
}

enum ProxiedRequest {
    Ok(Request<Incoming>, Span),
    Redirect(Response<String>),
    Err(ProxyError),
}

#[derive(Debug)]
struct SharedContext {
    pub router: Router,
    pub header_rewriter: RequestRewriter,
}

pub trait IoStream: AsyncRead + AsyncWrite + Unpin + Send {}

impl<S> IoStream for S where S: AsyncRead + AsyncWrite + Unpin + Send {}

#[derive(Debug, Clone)]
pub struct Connection {
    pub name: ServerName<'static>,
    pub port: u16,
    pub tls: bool,
}

async fn redirect(req: hyper::Request<Incoming>) -> Result<Response<String>, hyper::http::Error> {
    if let Ok(uri) = get_secure_uri(&req) {
        Response::builder()
            .header("Location", uri.to_string())
            .status(StatusCode::PERMANENT_REDIRECT)
            .body(String::new())
    } else {
        Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(String::from("TLS required\r\n"))
    }
}

fn get_secure_uri(req: &hyper::Request<Incoming>) -> anyhow::Result<Uri> {
    let mut parts = req.uri().clone().into_parts();
    if let Some(host) = req.headers().get(HOST) {
        parts.authority = Some(host.to_str()?.parse()?);
    }
    parts.scheme = Some(Scheme::HTTPS);
    Ok(Uri::from_parts(parts)?)
}
