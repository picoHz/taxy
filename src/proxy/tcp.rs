use super::{PortContextEvent, PortStatus, SocketState};
use crate::{config::port::PortEntry, error::Error};
use multiaddr::{Multiaddr, Protocol};
use std::{
    net::{IpAddr, SocketAddr},
    sync::Arc,
    time::SystemTime,
};
use tokio::net::{self, TcpSocket, TcpStream};
use tokio_rustls::{
    rustls::{client::ServerName, Certificate, ClientConfig, RootCertStore},
    TlsConnector,
};
use tracing::{debug, error, info, span, warn, Instrument, Level, Span};

#[derive(Debug)]
pub struct TcpPortContext {
    pub listen: SocketAddr,
    servers: Vec<Connection>,
    status: PortStatus,
    span: Span,
    tls_client_config: Option<Arc<ClientConfig>>,
    round_robin_counter: usize,
}

impl TcpPortContext {
    pub fn new(port: &PortEntry) -> Result<Self, Error> {
        let span = span!(Level::INFO, "proxy", listen = ?port.listen);
        let enter = span.clone();
        let _enter = enter.enter();

        let listen = multiaddr_to_tcp(&port.listen)?;

        if port.servers.is_empty() {
            return Err(Error::EmptyBackendServers);
        }

        let mut servers = Vec::new();
        for server in &port.servers {
            let server = multiaddr_to_host(&server.addr)?;
            servers.push(server);
        }

        let mut tls_client_config = None;
        let use_tls = servers.iter().any(|server| server.tls);
        if use_tls {
            let mut root_certs = RootCertStore::empty();
            match rustls_native_certs::load_native_certs() {
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
            let config = ClientConfig::builder()
                .with_safe_defaults()
                .with_root_certificates(root_certs)
                .with_no_client_auth();
            tls_client_config = Some(Arc::new(config));
        }

        Ok(Self {
            listen,
            servers,
            status: Default::default(),
            span,
            tls_client_config,
            round_robin_counter: 0,
        })
    }

    pub async fn setup(&mut self) -> Result<(), Error> {
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
            let config = ClientConfig::builder()
                .with_safe_defaults()
                .with_root_certificates(root_certs)
                .with_no_client_auth();
            self.tls_client_config = Some(Arc::new(config));
        }
        Ok(())
    }

    pub fn apply(&mut self, new: Self) {
        self.listen = new.listen;
        self.servers = new.servers;
        self.span = new.span;
        self.tls_client_config = new.tls_client_config;
    }

    pub fn event(&mut self, event: PortContextEvent) {
        match event {
            PortContextEvent::SokcetStateUpadted(state) => {
                if self.status.socket != state {
                    self.status.started_at = if state == SocketState::Listening {
                        Some(SystemTime::now())
                    } else {
                        None
                    };
                }
                self.status.socket = state;
            }
        }
    }

    pub fn status(&self) -> &PortStatus {
        &self.status
    }

    pub fn start_proxy(&mut self, stream: TcpStream) {
        let span = self.span.clone();
        let conn = self.servers[self.round_robin_counter % self.servers.len()].clone();
        let tls_client_config = self
            .tls_client_config
            .as_ref()
            .filter(|_| conn.tls)
            .cloned();
        tokio::spawn(
            async move {
                if let Err(err) = start(stream, conn, tls_client_config).await {
                    error!("{err}");
                }
            }
            .instrument(span),
        );
        self.round_robin_counter = self.round_robin_counter.wrapping_add(1);
    }
}

pub async fn start(
    mut stream: TcpStream,
    conn: Connection,
    tls_client_config: Option<Arc<ClientConfig>>,
) -> anyhow::Result<()> {
    let remote = stream.peer_addr()?;
    let local = stream.local_addr()?;

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

    let mut out = sock.connect(resolved).await?;
    debug!(%resolved, "connected");

    if let Some(config) = tls_client_config {
        debug!(%resolved, "tls handshake");
        let tls = TlsConnector::from(config);
        let mut out = tls.connect(conn.name, out).await?;
        tokio::io::copy_bidirectional(&mut stream, &mut out).await?;
    } else {
        tokio::io::copy_bidirectional(&mut stream, &mut out).await?;
    }

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

#[derive(Debug, Clone)]
pub struct Connection {
    pub name: ServerName,
    pub port: u16,
    pub tls: bool,
}
