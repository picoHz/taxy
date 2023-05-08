use super::{tls::TlsTermination, PortContextEvent, PortStatus, SocketState};
use crate::{
    config::{port::PortEntry, AppConfig},
    error::Error,
    keyring::Keyring,
};
use hyper::{client, server::conn::Http};
use multiaddr::{Multiaddr, Protocol};
use std::{
    net::{IpAddr, SocketAddr},
    sync::Arc,
    time::SystemTime,
};
use tokio::io::{AsyncRead, AsyncWrite, BufStream};
use tokio::net::{self, TcpSocket, TcpStream};
use tokio_rustls::{
    rustls::{client::ServerName, Certificate, ClientConfig, RootCertStore},
    TlsAcceptor, TlsConnector,
};
use tracing::{debug, error, info, span, warn, Instrument, Level, Span};

#[derive(Debug)]
pub struct HttpPortContext {
    pub listen: SocketAddr,
    servers: Vec<Connection>,
    status: PortStatus,
    span: Span,
    tls_termination: Option<TlsTermination>,
    tls_client_config: Option<Arc<ClientConfig>>,
    round_robin_counter: usize,
}

impl HttpPortContext {
    pub fn new(port: &PortEntry) -> Result<Self, Error> {
        let span = span!(Level::INFO, "proxy", resource_id = port.id, listen = ?port.listen);
        let enter = span.clone();
        let _enter = enter.enter();

        info!("initializing http proxy");

        let listen = multiaddr_to_tcp(&port.listen)?;

        if port.servers.is_empty() {
            return Err(Error::EmptyBackendServers);
        }

        let mut servers = Vec::new();
        for server in &port.servers {
            let server = multiaddr_to_host(&server.addr)?;
            servers.push(server);
        }

        let tls_termination = if let Some(tls) = &port.opts.tls_termination {
            Some(TlsTermination::new(tls)?)
        } else if port.listen.iter().any(|p| p == Protocol::Tls) {
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

    pub fn start_proxy(&mut self, stream: BufStream<TcpStream>) {
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

        tokio::spawn(
            async move {
                if let Err(err) = start(stream, conn, tls_client_config, tls_acceptor).await {
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
    if let Some(acceptor) = tls_acceptor {
        debug!(%remote, "server: tls handshake");
        stream = Box::new(acceptor.accept(stream).await?);
    }

    let service = hyper::service::service_fn(move |mut req| {
        let tls_client_config = tls_client_config.clone();
        let conn = conn.clone();

        let uri_string = format!(
            "http://{}:{}{}",
            match &conn.name {
                ServerName::DnsName(name) => name.as_ref().to_string(),
                ServerName::IpAddress(addr) => addr.to_string(),
                _ => unreachable!(),
            },
            conn.port,
            req.uri()
                .path_and_query()
                .map(|x| x.as_str())
                .unwrap_or("/")
        );
        let uri = uri_string.parse().unwrap();
        *req.uri_mut() = uri;

        async move {
            let sock = if resolved.is_ipv4() {
                TcpSocket::new_v4()
            } else {
                TcpSocket::new_v6()
            }?;

            let out = sock.connect(resolved).await?;
            debug!(%resolved, "connected");

            let mut http2 = false;

            let mut out: Box<dyn IoStream> = Box::new(out);
            if let Some(config) = tls_client_config {
                debug!(%resolved, "client: tls handshake");
                let tls = TlsConnector::from(config.clone());
                let tls_stream = tls.connect(conn.name, out).await?;
                http2 = tls_stream.get_ref().1.alpn_protocol() == Some(b"h2");
                out = Box::new(tls_stream);
            }

            let (mut sender, conn) = client::conn::Builder::new()
                .http2_only(http2)
                .handshake(out)
                .await?;
            tokio::task::spawn(async move {
                if let Err(err) = conn.await {
                    error!("Connection failed: {:?}", err);
                }
            });
            Result::<_, anyhow::Error>::Ok(sender.send_request(req).await?)
        }
    });

    tokio::task::spawn(async move {
        if let Err(err) = Http::new().serve_connection(stream, service).await {
            error!("Failed to serve the connection: {:?}", err);
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

trait IoStream: AsyncRead + AsyncWrite + Unpin + Send {}

impl<S> IoStream for S where S: AsyncRead + AsyncWrite + Unpin + Send {}

#[derive(Debug, Clone)]
pub struct Connection {
    pub name: ServerName,
    pub port: u16,
    pub tls: bool,
}

#[derive(Clone)]
pub struct TokioExecutor;

impl<F> hyper::rt::Executor<F> for TokioExecutor
where
    F: std::future::Future + Send + 'static,
    F::Output: Send + 'static,
{
    fn execute(&self, fut: F) {
        tokio::task::spawn(fut);
    }
}
