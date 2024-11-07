use super::{tls::TlsTermination, PortContextEvent, PortStatus, SocketState};
use crate::server::cert_list::CertList;
use std::{net::SocketAddr, sync::Arc, time::SystemTime};
use taxy_api::{error::Error, multiaddr::Multiaddr, proxy::ProxyKind};
use taxy_api::{port::PortEntry, proxy::ProxyEntry};
use tokio::{
    io::AsyncWriteExt,
    net::{self, TcpSocket, TcpStream},
};
use tokio::{
    io::{AsyncRead, AsyncWrite, BufStream},
    sync::Notify,
};
use tokio_rustls::rustls::pki_types::{IpAddr, ServerName};
use tokio_rustls::{
    rustls::{ClientConfig, RootCertStore},
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
    tls_client_config: Arc<ClientConfig>,
    stop_notifier: Arc<Notify>,
}

impl TcpPortContext {
    pub fn new(entry: &PortEntry) -> Result<Self, Error> {
        let span = span!(Level::INFO, "proxy", resource_id = entry.id.to_string(), listen = %entry.port.listen);
        let enter = span.clone();
        let _enter = enter.enter();

        info!("initializing tcp proxy");

        let listen = entry.port.listen.socket_addr()?;
        let tls_termination = if let Some(tls) = &entry.port.opts.tls_termination {
            Some(TlsTermination::new(tls, vec![])?)
        } else if entry.port.listen.is_tls() {
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
            tls_client_config: Arc::new(
                ClientConfig::builder()
                    .with_root_certificates(RootCertStore::empty())
                    .with_no_client_auth(),
            ),
            stop_notifier: Arc::new(Notify::new()),
        })
    }

    pub async fn setup(&mut self, certs: &CertList, proxies: Vec<ProxyEntry>) -> Result<(), Error> {
        let config = ClientConfig::builder()
            .with_root_certificates(certs.root_certs().clone())
            .with_no_client_auth();
        self.tls_client_config = Arc::new(config);

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

    pub fn start_proxy(&mut self, mut stream: BufStream<TcpStream>) {
        if self.servers.is_empty() {
            tokio::spawn(async move { stream.get_mut().shutdown().await });
            return;
        }

        let span = self.span.clone();
        let conn = self.servers[0].clone();
        let tls_client_config = if conn.tls {
            Some(self.tls_client_config.clone())
        } else {
            None
        };
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

    let (mut client_stream, server_stream) = tokio::io::duplex(MAX_BUFFER_SIZE);
    tokio::spawn(async move {
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
    });

    let name = match &conn.name {
        ServerName::DnsName(name) => name.as_ref().to_string(),
        ServerName::IpAddress(addr) => match addr {
            IpAddr::V4(addr) => std::net::Ipv4Addr::from(*addr).to_string(),
            IpAddr::V6(addr) => std::net::Ipv6Addr::from(*addr).to_string(),
        },
        _ => unreachable!(),
    };
    let host = format!("{}:{}", name, conn.port);

    let resolved = net::lookup_host(&host).await?.next().unwrap();
    debug!(host, %resolved);

    let sock = if resolved.is_ipv4() {
        TcpSocket::new_v4()
    } else {
        TcpSocket::new_v6()
    }?;

    info!(target: "taxy::access_log", remote = %remote, %local, target = host);

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

fn multiaddr_to_host(addr: &Multiaddr) -> Result<Connection, Error> {
    let tls = addr.is_tls();
    match (addr.ip_addr(), addr.host(), addr.port()) {
        (Ok(addr), _, Ok(port)) => Ok(Connection {
            name: ServerName::IpAddress(addr.into()),
            port,
            tls,
        }),
        (_, Ok(host), Ok(port)) => Ok(Connection {
            name: ServerName::try_from(host.as_str())
                .map_err(|_| Error::InvalidServerAddress { addr: addr.clone() })?
                .to_owned(),
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
    pub name: ServerName<'static>,
    pub port: u16,
    pub tls: bool,
}
