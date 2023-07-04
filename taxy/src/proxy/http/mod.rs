use self::route::Router;
use super::{tls::TlsTermination, PortContextEvent};
use crate::{
    proxy::http::compression::{is_compressed, CompressionStream},
    server::cert_list::CertList,
};
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
use taxy_api::{port::PortEntry, site::SiteEntry};
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

mod compression;
mod filter;
mod header;
mod route;
mod upgrade;

const MAX_BUFFER_SIZE: usize = 4096;

#[derive(Debug)]
pub struct HttpPortContext {
    pub listen: SocketAddr,
    status: PortStatus,
    span: Span,
    tls_termination: Option<TlsTermination>,
    tls_client_config: Option<Arc<ClientConfig>>,
    router: Arc<Router>,
    round_robin_counter: usize,
    stop_notifier: Arc<Notify>,
}

impl HttpPortContext {
    pub fn new(entry: &PortEntry) -> Result<Self, Error> {
        let span = span!(Level::INFO, "proxy", resource_id = entry.id, listen = ?entry.port.listen);
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
            router: Arc::new(Default::default()),
            round_robin_counter: 0,
            stop_notifier: Arc::new(Notify::new()),
        })
    }

    pub async fn setup(&mut self, certs: &CertList, sites: Vec<SiteEntry>) -> Result<(), Error> {
        self.router = Arc::new(Router::new(sites));

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
            round_robin_counter: self.round_robin_counter,
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

        let header_rewriter = HeaderRewriter::builder()
            .trust_upstream_headers(false)
            .use_std_forwarded(true)
            .set_via(HeaderValue::from_static("taxy"))
            .build();

        let stop_notifier = self.stop_notifier.clone();
        let router = self.router.clone();
        let round_robin_counter = self.round_robin_counter;

        tokio::spawn(
            async move {
                if let Err(err) = start(
                    stream,
                    tls_client_config,
                    tls_acceptor,
                    header_rewriter,
                    router,
                    round_robin_counter,
                    stop_notifier,
                )
                .await
                {
                    error!("{err}");
                }
            }
            .instrument(span),
        );
        self.round_robin_counter = self.round_robin_counter.wrapping_add(1);
    }
}

pub async fn start(
    mut stream: BufStream<TcpStream>,
    tls_client_config: Option<Arc<ClientConfig>>,
    tls_acceptor: Option<TlsAcceptor>,
    header_rewriter: HeaderRewriter,
    router: Arc<Router>,
    round_robin_counter: usize,
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

    let router = router.clone();
    let service = hyper::service::service_fn(move |mut req| {
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

        let mut hostname = String::new();
        let mut host = String::new();

        let mut span = Span::current();

        let mut client_tls = false;
        if let Some((route, res, resource_id)) = router.get_route(&req) {
            span = span!(Level::INFO, "proxy", resource_id);

            let mut parts = Parts::default();

            parts.path_and_query = if let Some(query) = req.uri().query() {
                format!("{}?{}", res.uri.path(), query).parse().ok()
            } else {
                res.uri.path_and_query().cloned()
            };

            if !route.servers.is_empty() {
                let server = &route.servers[round_robin_counter % route.servers.len()];

                parts.scheme = Some(if server.url.scheme() == "http" {
                    Scheme::HTTP
                } else {
                    Scheme::HTTPS
                });
                client_tls = parts.scheme == Some(Scheme::HTTPS);

                hostname = server
                    .url
                    .host()
                    .map(|host| host.to_string())
                    .unwrap_or_default();
                host = format!(
                    "{}:{}",
                    hostname,
                    server.url.port_or_known_default().unwrap_or_default()
                );

                parts.authority = host.parse().ok();

                if let Some(req_host) = req.headers_mut().get_mut(HOST) {
                    *req_host = HeaderValue::from_str(&host).unwrap();
                }
            }

            if let Ok(uri) = Uri::from_parts(parts) {
                *req.uri_mut() = uri;
            }
        }

        header_rewriter.pre_process(req.headers_mut(), remote.ip());
        header_rewriter.post_process(req.headers_mut());

        let span_cloned = span.clone();
        async move {
            if hostname.is_empty() || domain_fronting {
                let mut res = hyper::Response::new(hyper::Body::empty());
                *res.status_mut() = hyper::StatusCode::BAD_GATEWAY;
                return Ok::<_, anyhow::Error>(res);
            }

            let resolved = net::lookup_host(&host).await?.next().unwrap();
            debug!(host, %resolved);

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
                let tls_stream = tls
                    .connect(ServerName::try_from(hostname.as_str()).unwrap(), out)
                    .await?;
                client_http2 = tls_stream.get_ref().1.alpn_protocol() == Some(b"h2");
                out = Box::new(tls_stream);
            }

            if upgrade {
                return upgrade::connect(req, out).instrument(span).await;
            }

            let (mut sender, conn) = client::conn::Builder::new()
                .http2_only(client_http2)
                .handshake(out)
                .await
                .map_err(|err| {
                    println!("cerr: {:?}", err);
                    err
                })?;

            tokio::task::spawn(
                async move {
                    if let Err(err) = conn.await {
                        error!("Connection failed: {:?}", err);
                    }
                }
                .instrument(span),
            );

            let accept_brotli = req
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
                            let stream = CompressionStream::new(body);
                            return Response::from_parts(parts, hyper::Body::wrap_stream(stream));
                        }
                    }
                }

                Response::from_parts(parts, body)
            })
        }
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

pub trait IoStream: AsyncRead + AsyncWrite + Unpin + Send {}

impl<S> IoStream for S where S: AsyncRead + AsyncWrite + Unpin + Send {}

#[derive(Debug, Clone)]
pub struct Connection {
    pub name: ServerName,
    pub port: u16,
    pub tls: bool,
}
