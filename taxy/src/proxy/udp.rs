use super::{PortContextEvent, PortStatus, SocketState};
use hickory_resolver::config::LookupIpStrategy;
use hickory_resolver::name_server::{GenericConnector, TokioRuntimeProvider};
use hickory_resolver::system_conf::read_system_conf;
use hickory_resolver::AsyncResolver;
use std::time::{Duration, Instant};
use std::{net::SocketAddr, time::SystemTime};
use taxy_api::{error::Error, multiaddr::Multiaddr, proxy::ProxyKind};
use taxy_api::{port::PortEntry, proxy::ProxyEntry};
use tokio_rustls::rustls::pki_types::ServerName;
use tracing::{info, span, Level, Span};

const DNS_LOOKUP_RETRY_INTERVAL: Duration = Duration::from_secs(5);

#[derive(Debug)]
pub struct UdpPortContext {
    pub listen: SocketAddr,
    servers: Vec<Connection>,
    status: PortStatus,
    span: Span,
    resolver: AsyncResolver<GenericConnector<TokioRuntimeProvider>>,
}

impl UdpPortContext {
    pub fn new(entry: &PortEntry) -> Result<Self, Error> {
        let span = span!(Level::INFO, "proxy", resource_id = entry.id.to_string(), listen = %entry.port.listen);
        let enter = span.clone();
        let _enter = enter.enter();

        info!("initializing udp proxy");

        let (conf, mut opts) = read_system_conf().unwrap_or_default();
        opts.ip_strategy = LookupIpStrategy::Ipv4AndIpv6;
        let resolver = AsyncResolver::tokio(conf, opts);

        let listen = entry.port.listen.socket_addr()?;
        Ok(Self {
            listen,
            servers: Default::default(),
            status: Default::default(),
            span,
            resolver,
        })
    }

    pub async fn setup(&mut self, proxies: Vec<ProxyEntry>) -> Result<(), Error> {
        for proxy in proxies {
            if let ProxyKind::Udp(proxy) = proxy.proxy.kind {
                for server in proxy.upstream_servers {
                    let server = multiaddr_to_host(&server.addr)?;
                    self.servers.push(server);
                }
            }
        }
        Ok(())
    }

    pub fn apply(&mut self, new: Self) {
        *self = new;
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

    pub fn reset(&mut self) {}

    pub async fn target_addrs(&mut self) -> Vec<SocketAddr> {
        let mut addrs = Vec::new();
        for server in &mut self.servers {
            if !server.ttl.elapsed().is_zero() {
                let resolved = self.resolver.lookup_ip(server.name.to_str().as_ref()).await;
                let (resolved, ttl): (anyhow::Result<SocketAddr>, Instant) = match resolved {
                    Ok(addrs) => {
                        if let Some(addr) = addrs.iter().next() {
                            (Ok(SocketAddr::new(addr, server.port)), addrs.valid_until())
                        } else {
                            (
                                Err(anyhow::anyhow!(
                                    "no IP address found for {}",
                                    server.name.to_str()
                                )),
                                addrs.valid_until(),
                            )
                        }
                    }
                    Err(e) => (Err(e.into()), Instant::now() + DNS_LOOKUP_RETRY_INTERVAL),
                };
                server.ttl = ttl;
                match resolved {
                    Ok(addr) => {
                        server.addr = Some(addr);
                    }
                    Err(e) => {
                        self.span.in_scope(|| {
                            tracing::error!("failed to resolve {}: {}", server.name.to_str(), e);
                        });
                    }
                }
            }
            if let Some(addr) = server.addr {
                addrs.push(addr);
            }
        }
        addrs
    }
}

fn multiaddr_to_host(addr: &Multiaddr) -> Result<Connection, Error> {
    match (addr.ip_addr(), addr.host(), addr.port()) {
        (Ok(addr), _, Ok(port)) => Ok(Connection {
            name: ServerName::IpAddress(addr.into()),
            port,
            addr: None,
            ttl: Instant::now(),
        }),
        (_, Ok(host), Ok(port)) => Ok(Connection {
            name: ServerName::try_from(host.as_str())
                .map_err(|_| Error::InvalidServerAddress { addr: addr.clone() })?
                .to_owned(),
            port,
            addr: None,
            ttl: Instant::now(),
        }),
        _ => Err(Error::InvalidServerAddress { addr: addr.clone() }),
    }
}

#[derive(Debug, Clone)]
pub struct Connection {
    pub name: ServerName<'static>,
    pub port: u16,
    pub addr: Option<SocketAddr>,
    pub ttl: Instant,
}
