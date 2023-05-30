use super::{tls::TlsTermination, PortContextEvent, PortStatus, SocketState};
use crate::{
    config::{port::PortEntry, AppConfig},
    error::Error,
    keyring::Keyring,
};
use hyper::{
    client,
    header::{HOST, UPGRADE},
    http::HeaderValue,
    server::conn::Http,
};
use multiaddr::{Multiaddr, Protocol};
use std::{
    net::{IpAddr, SocketAddr},
    sync::Arc,
    time::SystemTime,
};
use tokio::{
    io::AsyncWriteExt,
    net::{self, TcpSocket, TcpStream},
};
use tokio::{
    io::{AsyncRead, AsyncWrite, BufStream},
    sync::Notify,
};
use tokio_rustls::{
    rustls::{client::ServerName, Certificate, ClientConfig, RootCertStore},
    TlsAcceptor, TlsConnector,
};
use tracing::{debug, error, info, span, warn, Instrument, Level, Span};

mod header;
mod upgrade;

use header::HeaderRewriter;

#[derive(Debug)]
pub struct HttpPortContext {
    pub listen: SocketAddr,
    servers: Vec<Connection>,
    status: PortStatus,
    span: Span,
    tls_termination: Option<TlsTermination>,
    tls_client_config: Option<Arc<ClientConfig>>,
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

        let mut servers = Vec::new();
        for server in &entry.port.opts.upstream_servers {
            let server = multiaddr_to_host(&server.addr)?;
            servers.push(server);
        }

        let tls_termination = if let Some(tls) = &entry.port.opts.tls_termination {
            let alpn = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
            Some(TlsTermination::new(tls, alpn)?)
        } else if entry.port.listen.iter().any(|p| p == Protocol::Tls) {
            return Err(Error::TlsTerminationConfigMissing);
        } else {
            None
        };

        Ok(Self {
            listen,
            servers,
            status: Default::default(),
            span,
            tls_termination,
            tls_client_config: None,
            round_robin_counter: 0,
            stop_notifier: Arc::new(Notify::new()),
        })
    }

    pub async fn prepare(&mut self, _config: &AppConfig) -> Result<(), Error> {
        let use_tls = self.servers.iter().any(|server| server.tls);
        if self.tls_client_config.is_none() && use_tls {
            let mut root_certs = RootCertStore::empty();
            if let Ok(certs) =
                tokio::task::spawn_blocking(rustls_native_certs::load_native_certs).await
            {
                match certs {
                    Ok(certs) => {
                        for certs in certs {
                            if let Err(err) = root_certs.add(&Certificate(certs.0)) {
                                warn!("failed to add native certs: {err}");
                            }
                        }
                    }
                    Err(err) => {
                        warn!("failed to load native certs: {err}");
                    }
                }
            }
            let mut config = ClientConfig::builder()
                .with_safe_defaults()
                .with_root_certificates(root_certs)
                .with_no_client_auth();
            config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
            self.tls_client_config = Some(Arc::new(config));
        }

        Ok(())
    }

    pub async fn setup(&mut self, keyring: &Keyring) -> Result<(), Error> {
        if let Some(tls) = &mut self.tls_termination {
            self.status.state.tls = Some(tls.setup(keyring).await);
        }
        Ok(())
    }

    pub async fn refresh(&mut self, certs: &Keyring) -> Result<(), Error> {
        if let Some(tls) = &mut self.tls_termination {
            self.status.state.tls = Some(tls.refresh(certs).await);
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
            PortContextEvent::SiteTableUpdated(sites) => {
                println!("sites: {:?}", sites);
            }
        }
    }

    pub fn status(&self) -> &PortStatus {
        &self.status
    }

    pub fn reset(&mut self) {
        self.stop_notifier.notify_waiters();
    }

    pub fn start_proxy(&mut self, mut stream: BufStream<TcpStream>) {
        if self.servers.is_empty() {
            tokio::spawn(async move { stream.get_mut().shutdown().await });
            return;
        }

        let span = self.span.clone();
        let conn = self.servers[self.round_robin_counter % self.servers.len()].clone();

        let tls_client_config = self
            .tls_client_config
            .as_ref()
            .filter(|_| conn.tls)
            .cloned();
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

        tokio::spawn(
            async move {
                if let Err(err) = start(
                    stream,
                    conn,
                    tls_client_config,
                    tls_acceptor,
                    header_rewriter,
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
    stream: BufStream<TcpStream>,
    conn: Connection,
    tls_client_config: Option<Arc<ClientConfig>>,
    tls_acceptor: Option<TlsAcceptor>,
    header_rewriter: HeaderRewriter,
    stop_notifier: Arc<Notify>,
) -> anyhow::Result<()> {
    let remote = stream.get_ref().peer_addr()?;
    let local = stream.get_ref().local_addr()?;

    let host = match conn.name.clone() {
        ServerName::DnsName(name) => format!("{}:{}", name.as_ref(), conn.port),
        ServerName::IpAddress(addr) => format!("{}:{}", addr, conn.port),
        _ => unreachable!(),
    };

    let resolved = net::lookup_host(&host).await?.next().unwrap();
    debug!(host, %resolved);

    info!(target: "taxy::access_log", remote = %remote, %local, %resolved);

    let mut stream: Box<dyn IoStream> = Box::new(stream);
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

    let stop_notifier_clone = stop_notifier.clone();
    let service = hyper::service::service_fn(move |mut req| {
        let tls_client_config = tls_client_config.clone();
        let conn = conn.clone();

        let upgrade = req.headers().contains_key(UPGRADE);
        let hostname = match &conn.name {
            ServerName::DnsName(name) => name.as_ref().to_string(),
            ServerName::IpAddress(addr) => addr.to_string(),
            _ => unreachable!(),
        };
        let host = format!("{}:{}", hostname, conn.port);
        let domain_fronting = match (&sni, req.headers().get(HOST).and_then(|h| h.to_str().ok())) {
            (Some(sni), Some(header)) => sni.eq_ignore_ascii_case(header),
            _ => false,
        };

        let uri_string = format!(
            "http://{}{}",
            host,
            req.uri()
                .path_and_query()
                .map(|x| x.as_str())
                .unwrap_or("/")
        );
        let uri = uri_string.parse().unwrap();
        *req.uri_mut() = uri;
        if let Some(req_host) = req.headers_mut().get_mut(HOST) {
            *req_host = HeaderValue::from_str(&host).unwrap();
        }

        header_rewriter.pre_process(req.headers_mut(), remote.ip());
        header_rewriter.post_process(req.headers_mut());

        let stop_notifier = stop_notifier_clone.clone();
        async move {
            if domain_fronting {
                debug!(%host, "domain fronting detected");
                let mut res = hyper::Response::new(hyper::Body::empty());
                *res.status_mut() = hyper::StatusCode::BAD_GATEWAY;
                return Ok::<_, anyhow::Error>(res);
            }

            let sock = if resolved.is_ipv4() {
                TcpSocket::new_v4()
            } else {
                TcpSocket::new_v6()
            }?;

            let out = sock.connect(resolved).await?;
            debug!(%resolved, "connected");

            let mut client_http2 = false;

            let mut out: Box<dyn IoStream> = Box::new(out);
            if let Some(config) = tls_client_config {
                debug!(%resolved, "client: tls handshake");
                let tls = TlsConnector::from(config.clone());
                let tls_stream = tls.connect(conn.name, out).await?;
                client_http2 = tls_stream.get_ref().1.alpn_protocol() == Some(b"h2");
                out = Box::new(tls_stream);
            }

            if upgrade {
                return upgrade::connect(req, out, stop_notifier.clone()).await;
            }

            let (mut sender, conn) = client::conn::Builder::new()
                .http2_only(client_http2)
                .handshake(out)
                .await
                .map_err(|err| {
                    println!("cerr: {:?}", err);
                    err
                })?;

            tokio::task::spawn(async move {
                tokio::select! {
                    result = conn => {
                        if let Err(err) = result {
                            error!("Connection failed: {:?}", err);
                        }
                    },
                    _ = stop_notifier.notified() => {
                        debug!("stop");
                    },
                }
            });

            Result::<_, anyhow::Error>::Ok(sender.send_request(req).await?)
        }
    });

    tokio::task::spawn(async move {
        let http = Http::new()
            .http2_only(server_http2)
            .serve_connection(stream, service)
            .with_upgrades();
        tokio::select! {
            result = http => {
                if let Err(err) = result {
                    error!("Failed to serve the connection: {:?}", err);
                }
            },
            _ = stop_notifier.notified() => {
                debug!("stop");
            },
        }
    });

    debug!(%resolved, "eof");
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

fn multiaddr_to_host(addr: &Multiaddr) -> Result<Connection, Error> {
    let stack = addr.iter().collect::<Vec<_>>();
    let tls = stack.contains(&Protocol::Tls);
    match stack[..] {
        [Protocol::Ip4(addr), Protocol::Tcp(port), ..] if port > 0 => Ok(Connection {
            name: ServerName::IpAddress(IpAddr::V4(addr)),
            port,
            tls,
        }),
        [Protocol::Ip6(addr), Protocol::Tcp(port), ..] if port > 0 => Ok(Connection {
            name: ServerName::IpAddress(IpAddr::V6(addr)),
            port,
            tls,
        }),
        [Protocol::Dns(ref name), Protocol::Tcp(port), ..] if port > 0 => Ok(Connection {
            name: ServerName::try_from(name.as_ref())
                .map_err(|_| Error::InvalidServerAddress { addr: addr.clone() })?,
            port,
            tls,
        }),
        _ => Err(Error::InvalidServerAddress { addr: addr.clone() }),
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
