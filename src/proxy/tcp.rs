use super::{PortContextEvent, PortStatus, SocketState};
use crate::{config::port::PortEntry, error::Error};
use multiaddr::{Multiaddr, Protocol};
use std::{net::SocketAddr, time::SystemTime};
use tokio::net::{self, TcpSocket, TcpStream};
use tracing::{debug, error, info, span, Instrument, Level, Span};

#[derive(Debug)]
pub struct TcpPortContext {
    pub listen: SocketAddr,
    servers: Vec<String>,
    status: PortStatus,
    span: Span,
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

        Ok(Self {
            listen,
            servers,
            status: Default::default(),
            span,
            round_robin_counter: 0,
        })
    }

    pub fn apply(&mut self, new: Self) {
        self.listen = new.listen;
        self.servers = new.servers;
        self.span = new.span;
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
        let server = self.servers[self.round_robin_counter % self.servers.len()].clone();
        tokio::spawn(
            async move {
                if let Err(err) = start(stream, server).await {
                    error!("{err}");
                }
            }
            .instrument(span),
        );
        self.round_robin_counter = self.round_robin_counter.wrapping_add(1);
    }
}

pub async fn start(mut stream: TcpStream, host: String) -> anyhow::Result<()> {
    let remote = stream.peer_addr()?;
    let local = stream.local_addr()?;

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

    tokio::io::copy_bidirectional(&mut stream, &mut out).await?;

    debug!(%resolved, "eof");
    Ok(())
}

pub fn multiaddr_to_tcp(addr: &Multiaddr) -> Result<SocketAddr, Error> {
    let stack = addr.iter().collect::<Vec<_>>();
    match &stack[..] {
        [Protocol::Ip4(addr), Protocol::Tcp(port)] if *port > 0 => {
            Ok(SocketAddr::new(std::net::IpAddr::V4(*addr), *port))
        }
        [Protocol::Ip6(addr), Protocol::Tcp(port)] if *port > 0 => {
            Ok(SocketAddr::new(std::net::IpAddr::V6(*addr), *port))
        }
        _ => Err(Error::InvalidListeningAddress { addr: addr.clone() }),
    }
}

pub fn multiaddr_to_host(addr: &Multiaddr) -> Result<String, Error> {
    let stack = addr.iter().collect::<Vec<_>>();
    match &stack[..] {
        [Protocol::Ip4(addr), Protocol::Tcp(port)] if *port > 0 => {
            Ok(SocketAddr::new(std::net::IpAddr::V4(*addr), *port).to_string())
        }
        [Protocol::Ip6(addr), Protocol::Tcp(port)] if *port > 0 => {
            Ok(SocketAddr::new(std::net::IpAddr::V6(*addr), *port).to_string())
        }
        [Protocol::Dns(addr), Protocol::Tcp(port)] if *port > 0 => Ok(format!("{}:{}", addr, port)),
        _ => Err(Error::InvalidServerAddress { addr: addr.clone() }),
    }
}
