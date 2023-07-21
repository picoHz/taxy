use self::route::Router;
use super::{tls::TlsTermination, PortContextEvent};
use crate::{
    proxy::http::{
        compression::{is_compressed, CompressionStream},
        error::ProxyError,
    },
    server::cert_list::CertList,
};
use arc_swap::{ArcSwap, Cache};
use futures::FutureExt;
use header::HeaderRewriter;
use hyper::{
    client,
    header::{HOST, UPGRADE},
    http::{
        uri::{Parts, Scheme},
        HeaderValue,
    },
    server::conn::Http,
    Response, Uri,
};
use multiaddr::{Multiaddr, Protocol};
use std::{net::SocketAddr, sync::Arc, time::SystemTime};
use taxy_api::error::Error;
use taxy_api::port::{PortStatus, SocketState};
use taxy_api::{port::PortEntry, site::ProxyEntry};
use tokio::net::{self, TcpSocket, TcpStream};
use tokio::{
    io::{AsyncRead, AsyncWrite, BufStream},
    sync::Notify,
};
use tokio_rustls::{
    rustls::{client::ServerName, ClientConfig},
    TlsAcceptor, TlsConnector,
};
use tracing::{debug, error, info, span, Instrument, Level, Span};
use warp::host::Authority;

mod compression;
mod error;
mod filter;
mod header;
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
    tls_client_config: Option<Arc<ClientConfig>>,
    shared: Arc<ArcSwap<SharedContext>>,
    stop_notifier: Arc<Notify>,
}

impl HttpPortContext {
    pub fn new(entry: &PortEntry) -> Result<Self, Error> {
        let span =
            span!(Level::INFO, "proxy", resource_id = ?entry.id, listen = ?entry.port.listen);
        let enter = span.clone();
        let _enter = enter.enter();

        info!("initializing http proxy");
        let listen = multiaddr_to_tcp(&entry.port.listen)?;

        let tls_termination = if let Some(tls) = &entry.port.opts.tls_termination {
            let alpn = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
            Some(TlsTermination::new(tls, alpn)?)
        } else if entry
            .port
            .listen
            .iter()
            .any(|p| matches!(p, Protocol::Tls) || matches!(p, Protocol::Https))
        {
            return Err(Error::TlsTerminationConfigMissing);
        } else {
            None
        };

        Ok(Self {
            listen,
            status: Default::default(),
            span,
            tls_termination,
            tls_client_config: None,
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
            .with_safe_defaults()
            .with_root_certificates(certs.root_certs().clone())
            .with_no_client_auth();
        self.tls_client_config = Some(Arc::new(config));

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

        tokio::spawn(
            async move {
                if let Err(err) = start(
                    stream,
                    tls_client_config,
                    tls_acceptor,
                    shared_cache,
                    stop_notifier,
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
    tls_client_config: Option<Arc<ClientConfig>>,
    tls_acceptor: Option<TlsAcceptor>,
    shared_cache: Cache<Arc<ArcSwap<SharedContext>>, Arc<SharedContext>>,
    stop_notifier: Arc<Notify>,
) -> anyhow::Result<()> {
    let remote = stream.get_ref().peer_addr()?;
    let local = stream.get_ref().local_addr()?;

    let (mut client_strem, server_stream) = tokio::io::duplex(MAX_BUFFER_SIZE);
    tokio::spawn(async move {
        tokio::select! {
            result = tokio::io::copy_bidirectional(&mut stream, &mut client_strem) => {
                if let Err(err) = result {
                    error!("{err}");
                }
            },
            _ = stop_notifier.notified() => {
                debug!("stop");
            },
        }
    });

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

    let mut shared_cache = shared_cache.clone();
    let service = hyper::service::service_fn(move |mut req| {
        let shared = shared_cache.load();
        let tls_client_config = tls_client_config.clone();
        let upgrade = req.headers().contains_key(UPGRADE);

        let header_host = req
            .headers()
            .get(HOST)
            .and_then(|h| h.to_str().ok().and_then(|host| host.split(':').next()));
        let domain_fronting = match (&sni, header_host) {
            (Some(sni), Some(header)) => !sni.eq_ignore_ascii_case(header),
            _ => false,
        };

        if domain_fronting {
            debug!("domain fronting detected");
        }

        let mut destination: Option<(ServerName, Authority)> = None;
        let mut span = Span::current();

        let mut client_tls = false;
        if let Some((route, res, resource_id)) = shared.router.get_route(&req) {
            span = span!(Level::INFO, "proxy", ?resource_id);

            let mut parts = Parts::default();

            parts.path_and_query = if let Some(query) = req.uri().query() {
                format!("{}?{}", res.uri.path(), query).parse().ok()
            } else {
                res.uri.path_and_query().cloned()
            };

            if !route.servers.is_empty() {
                let server = &route.servers[0];

                parts.scheme = Some(if server.url.scheme() == "http" {
                    Scheme::HTTP
                } else {
                    Scheme::HTTPS
                });
                client_tls = parts.scheme == Some(Scheme::HTTPS);

                let authority = server.authority.clone();
                req.headers_mut()
                    .insert(HOST, HeaderValue::from_str(authority.as_str()).unwrap());
                parts.authority = Some(authority);
                destination = Some((server.server_name.clone(), server.authority.clone()));
            }

            if let Ok(uri) = Uri::from_parts(parts) {
                *req.uri_mut() = uri;
            }
        }

        shared
            .header_rewriter
            .pre_process(req.headers_mut(), remote.ip());
        shared.header_rewriter.post_process(req.headers_mut());

        let span_cloned = span.clone();
        async move {
            if domain_fronting {
                return Err(ProxyError::InvalidHostName.into());
            }

            let (server_name, authority) = if let Some(destination) = destination {
                destination
            } else {
                return Err(ProxyError::InvalidHostName.into());
            };

            let resolved = net::lookup_host(authority.as_str())
                .await
                .map_err(|_| ProxyError::DnsLookupFailed)?
                .next()
                .ok_or(ProxyError::DnsLookupFailed)?;
            debug!(%authority, %resolved);

            info!(target: "taxy::access_log", remote = %remote, %local, %resolved);

            let sock = if resolved.is_ipv4() {
                TcpSocket::new_v4()
            } else {
                TcpSocket::new_v6()
            }?;

            let out = sock.connect(resolved).await?;
            debug!(%resolved, "connected");

            let mut client_http2 = false;

            let mut out: Box<dyn IoStream> = Box::new(out);
            if let Some(config) = tls_client_config.filter(|_| client_tls) {
                debug!(%resolved, "client: tls handshake");
                let tls = TlsConnector::from(config.clone());
                let tls_stream = tls.connect(server_name, out).await?;
                client_http2 = tls_stream.get_ref().1.alpn_protocol() == Some(b"h2");
                out = Box::new(tls_stream);
            }

            if upgrade {
                return upgrade::connect(req, out).instrument(span).await;
            }

            let (mut sender, conn) = client::conn::Builder::new()
                .http2_only(client_http2)
                .http2_max_frame_size(Some(HTTP2_MAX_FRAME_SIZE as u32))
                .handshake(out)
                .await?;

            tokio::task::spawn(
                async move {
                    if let Err(err) = conn.await {
                        error!("Connection failed: {:?}", err);
                    }
                }
                .instrument(span),
            );

            let accept_brotli = client_http2
                && req
                    .headers()
                    .get(hyper::header::ACCEPT_ENCODING)
                    .map(|value| value.to_str().unwrap_or_default().contains("br"))
                    .unwrap_or_default();

            let result = Result::<_, anyhow::Error>::Ok(sender.send_request(req).await?);
            result.map(|res| {
                let (mut parts, body) = res.into_parts();

                let is_compressed = parts
                    .headers
                    .get(hyper::header::CONTENT_TYPE)
                    .map(|value| is_compressed(value.as_bytes()))
                    .unwrap_or_default();

                if !is_compressed {
                    let encoding = parts.headers.entry(hyper::header::CONTENT_ENCODING);
                    if let hyper::header::Entry::Vacant(entry) = encoding {
                        if accept_brotli {
                            entry.insert(HeaderValue::from_static("br"));
                            parts.headers.remove(hyper::header::CONTENT_LENGTH);
                            let stream = CompressionStream::new(body, HTTP2_MAX_FRAME_SIZE);
                            return Response::from_parts(parts, hyper::Body::wrap_stream(stream));
                        }
                    }
                }

                Response::from_parts(parts, body)
            })
        }
        .map(error::map_response)
        .instrument(span_cloned)
    });

    tokio::task::spawn(async move {
        let http = Http::new()
            .http2_only(server_http2)
            .serve_connection(stream, service)
            .with_upgrades();
        if let Err(err) = http.await {
            error!("Failed to serve the connection: {:?}", err);
        }
    });

    Ok(())
}

fn multiaddr_to_tcp(addr: &Multiaddr) -> Result<SocketAddr, Error> {
    let stack = addr.iter().collect::<Vec<_>>();
    match &stack[..] {
        [Protocol::Ip4(addr), Protocol::Tcp(port), ..] if *port > 0 => {
            Ok(SocketAddr::new(std::net::IpAddr::V4(*addr), *port))
        }
        [Protocol::Ip6(addr), Protocol::Tcp(port), ..] if *port > 0 => {
            Ok(SocketAddr::new(std::net::IpAddr::V6(*addr), *port))
        }
        _ => Err(Error::InvalidListeningAddress { addr: addr.clone() }),
    }
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
    pub name: ServerName,
    pub port: u16,
    pub tls: bool,
}
