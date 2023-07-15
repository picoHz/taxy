use super::{tls::TlsTermination, PortContextEvent, PortStatus, SocketState};
use crate::server::{cert_list::CertList, health_checker::HealthChecker};
use multiaddr::{Multiaddr, Protocol};
use std::{
    net::{IpAddr, SocketAddr},
    sync::Arc,
    time::SystemTime,
};
use taxy_api::{error::Error, site::ProxyKind};
use taxy_api::{port::PortEntry, site::ProxyEntry};
use tokio::{
    io::AsyncWriteExt,
    net::{self, TcpSocket, TcpStream},
};
use tokio::{
    io::{AsyncRead, AsyncWrite, BufStream},
    sync::Notify,
};
use tokio_rustls::{
    rustls::{client::ServerName, ClientConfig},
    TlsAcceptor, TlsConnector,
};
use tracing::{debug, error, info, span, Instrument, Level, Span};

const MAX_BUFFER_SIZE: usize = 4096;

#[derive(Debug)]
pub struct TcpPortContext {
    pub listen: SocketAddr,
    servers: Vec<Connection>,
    status: PortStatus,
    span: Span,
    tls_termination: Option<TlsTermination>,
    tls_client_config: Option<Arc<ClientConfig>>,
    health_checker: HealthChecker,
    round_robin_counter: usize,
    stop_notifier: Arc<Notify>,
}

impl TcpPortContext {
    pub fn new(entry: &PortEntry) -> Result<Self, Error> {
        let span = span!(Level::INFO, "proxy", resource_id = entry.id, listen = ?entry.port.listen);
        let enter = span.clone();
        let _enter = enter.enter();

        info!("initializing tcp proxy");

        let listen = multiaddr_to_tcp(&entry.port.listen)?;
        let tls_termination = if let Some(tls) = &entry.port.opts.tls_termination {
            Some(TlsTermination::new(tls, vec![])?)
        } else if entry.port.listen.iter().any(|p| p == Protocol::Tls) {
            return Err(Error::TlsTerminationConfigMissing);
        } else {
            None
        };

        Ok(Self {
            listen,
            servers: Default::default(),
            status: Default::default(),
            span,
            tls_termination,
            tls_client_config: None,
            health_checker: HealthChecker::new(),
            round_robin_counter: 0,
            stop_notifier: Arc::new(Notify::new()),
        })
    }

    pub async fn setup(&mut self, certs: &CertList, proxies: Vec<ProxyEntry>) -> Result<(), Error> {
        let config = ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(certs.root_certs().clone())
            .with_no_client_auth();
        self.tls_client_config = Some(Arc::new(config));

        self.health_checker.update(&proxies);

        for proxy in proxies {
            if let ProxyKind::Tcp(proxy) = proxy.proxy.kind {
                for server in proxy.upstream_servers {
                    let server = multiaddr_to_host(&server.addr)?;
                    self.servers.push(server);
                }
            }
        }

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

    pub fn run_healthchecks(&mut self) {}

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

        let stop_notifier = self.stop_notifier.clone();

        tokio::spawn(
            async move {
                if let Err(err) =
                    start(stream, conn, tls_client_config, tls_acceptor, stop_notifier).await
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
    conn: Connection,
    tls_client_config: Option<Arc<ClientConfig>>,
    tls_acceptor: Option<TlsAcceptor>,
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

    let host = match conn.name.clone() {
        ServerName::DnsName(name) => format!("{}:{}", name.as_ref(), conn.port),
        ServerName::IpAddress(addr) => format!("{}:{}", addr, conn.port),
        _ => unreachable!(),
    };

    let resolved = net::lookup_host(&host).await?.next().unwrap();
    debug!(host, %resolved);

    let sock = if resolved.is_ipv4() {
        TcpSocket::new_v4()
    } else {
        TcpSocket::new_v6()
    }?;

    info!(target: "taxy::access_log", remote = %remote, %local, %resolved);

    let out = sock.connect(resolved).await?;
    debug!(%resolved, "connected");

    let mut stream: Box<dyn IoStream> = Box::new(server_stream);
    if let Some(acceptor) = tls_acceptor {
        debug!(%remote, "server: tls handshake");
        stream = Box::new(acceptor.accept(stream).await?);
    }

    let mut out: Box<dyn IoStream> = Box::new(out);
    if let Some(config) = tls_client_config {
        debug!(%resolved, "client: tls handshake");
        let tls = TlsConnector::from(config);
        out = Box::new(tls.connect(conn.name, out).await?);
    }

    if let Err(err) = tokio::io::copy_bidirectional(&mut stream, &mut out).await {
        error!("{err}");
    }

    stream.shutdown().await?;
    out.shutdown().await?;

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
    let tls = stack.last() == Some(&Protocol::Tls);
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
