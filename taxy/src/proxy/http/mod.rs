use self::{
    error::{map_response, ProxyError},
    pool::ConnectionPool,
    route::Router,
};
use super::{tls::TlsTermination, PortContextEvent};
use crate::server::cert_list::CertList;
use arc_swap::{ArcSwap, Cache};
use header::HeaderRewriter;
use hyper::{
    header::HOST,
    http::{
        uri::{Parts, Scheme},
        HeaderValue,
    },
    server::conn::Http,
    service::service_fn,
    Response, StatusCode, Uri,
};
use std::{net::SocketAddr, sync::Arc, time::SystemTime};
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

mod compression;
mod error;
mod filter;
mod header;
mod hyper_tls;
mod pool;
mod route;
mod upgrade;

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

    pub async fn setup(&mut self, certs: &CertList, proxies: Vec<ProxyEntry>) -> Result<(), Error> {
        self.shared.store(Arc::new(SharedContext {
            router: Router::new(proxies),
            header_rewriter: HeaderRewriter::builder()
                .trust_upstream_headers(false)
                .use_std_forwarded(true)
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
            PortContextEvent::SocketStateUpadted(state) => {
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
                if let Err(err) = Http::new()
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

    if let Some(acceptor) = tls_acceptor {
        debug!(%remote, "server: tls handshake");
        let accepted = acceptor.accept(stream).await?;
        let tls_conn = &accepted.get_ref().1;
        server_http2 = tls_conn.alpn_protocol() == Some(b"h2");
        sni = tls_conn.server_name().map(|sni| sni.to_string());
        stream = Box::new(accepted);
    }

    let pool = Arc::new(ConnectionPool::new(tls_client_config));
    let mut shared_cache = shared_cache.clone();
    let span_cloned = span.clone();
    let service = hyper::service::service_fn(move |mut req| {
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

        let action = format!("{} {}", req.method().as_str(), req.uri());
        let pool = pool.clone();
        let shared = shared_cache.load();

        let req = if domain_fronting {
            Err(ProxyError::DomainFrontingDetected)
        } else if let Some((route, res, _)) = shared.router.get_route(&req) {
            let mut parts = Parts::default();

            parts.path_and_query = if let Some(query) = req.uri().query() {
                format!("{}?{}", res.uri.path(), query).parse().ok()
            } else {
                res.uri.path_and_query().cloned()
            };

            if let Some(server) = route.servers.first() {
                parts.scheme = Some(match server.url.scheme() {
                    "https" | "wss" => Scheme::HTTPS,
                    _ => Scheme::HTTP,
                });

                let authority = server.authority.clone();
                req.headers_mut()
                    .insert(HOST, HeaderValue::from_str(authority.as_str()).unwrap());
                parts.authority = Some(authority);
            }

            if let Ok(uri) = Uri::from_parts(parts) {
                *req.uri_mut() = uri;
            }

            info!(target: "taxy::access_log", remote = %remote, %local, action, target = %req.uri());

            shared
                .header_rewriter
                .pre_process(req.headers_mut(), remote.ip());
            shared.header_rewriter.post_process(req.headers_mut());
            Ok(req)
        } else {
            Err(ProxyError::NoRouteFound)
        };

        async move {
            map_response(match req {
                Ok(req) => pool.request(req).await,
                Err(err) => Err(err.into()),
            })
        }
        .instrument(span)
    });

    tokio::task::spawn(
        async move {
            let http = Http::new()
                .http2_only(server_http2)
                .serve_connection(stream, service)
                .with_upgrades();
            if let Err(err) = http.await {
                error!("Failed to serve the connection: {:?}", err);
            }
        }
        .instrument(span.clone()),
    );

    Ok(())
}

#[derive(Debug)]
struct SharedContext {
    pub router: Router,
    pub header_rewriter: HeaderRewriter,
}

pub trait IoStream: AsyncRead + AsyncWrite + Unpin + Send {}

impl<S> IoStream for S where S: AsyncRead + AsyncWrite + Unpin + Send {}

#[derive(Debug, Clone)]
pub struct Connection {
    pub name: ServerName<'static>,
    pub port: u16,
    pub tls: bool,
}

async fn redirect(
    req: hyper::Request<hyper::Body>,
) -> Result<Response<hyper::Body>, hyper::http::Error> {
    if let Ok(uri) = get_secure_uri(&req) {
        Response::builder()
            .header("Location", uri.to_string())
            .status(StatusCode::PERMANENT_REDIRECT)
            .body(hyper::Body::empty())
    } else {
        Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(hyper::Body::from("TLS required\r\n"))
    }
}

fn get_secure_uri(req: &hyper::Request<hyper::Body>) -> anyhow::Result<Uri> {
    let mut parts = req.uri().clone().into_parts();
    if let Some(host) = req.headers().get(HOST) {
        parts.authority = Some(host.to_str()?.parse()?);
    }
    parts.scheme = Some(Scheme::HTTPS);
    Ok(Uri::from_parts(parts)?)
}
