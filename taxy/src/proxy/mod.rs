use self::{http::HttpPortContext, tcp::TcpPortContext, udp::UdpPortContext};
use crate::server::cert_list::CertList;
use once_cell::sync::OnceCell;
use taxy_api::error::Error;
use taxy_api::multiaddr::Multiaddr;
use taxy_api::port::{PortStatus, SocketState};
use taxy_api::{
    port::{Port, PortEntry},
    proxy::ProxyEntry,
};

pub mod http;
pub mod tcp;
pub mod tls;
pub mod udp;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PortContextEvent {
    SocketStateUpdated(SocketState),
}

#[derive(Debug)]
pub struct PortContext {
    pub entry: PortEntry,
    pub kind: PortContextKind,
}

impl PortContext {
    pub fn new(entry: PortEntry) -> Result<Self, Error> {
        let kind = if entry.port.listen.is_udp() {
            PortContextKind::Udp(UdpPortContext::new(&entry)?)
        } else if entry.port.listen.is_http() {
            PortContextKind::Http(HttpPortContext::new(&entry)?)
        } else {
            PortContextKind::Tcp(TcpPortContext::new(&entry)?)
        };
        Ok(Self { entry, kind })
    }

    pub fn reserved() -> Self {
        Self {
            entry: PortEntry {
                id: "reserved".parse().unwrap(),
                port: Port {
                    active: true,
                    name: String::new(),
                    listen: Multiaddr::default(),
                    opts: Default::default(),
                },
            },
            kind: PortContextKind::Reserved,
        }
    }

    pub fn entry(&self) -> &PortEntry {
        &self.entry
    }

    pub fn kind(&self) -> &PortContextKind {
        &self.kind
    }

    pub fn kind_mut(&mut self) -> &mut PortContextKind {
        &mut self.kind
    }

    pub async fn setup(
        &mut self,
        ports: &[PortEntry],
        certs: &CertList,
        proxies: Vec<ProxyEntry>,
    ) -> Result<(), Error> {
        match &mut self.kind {
            PortContextKind::Tcp(ctx) => ctx.setup(certs, proxies).await,
            PortContextKind::Http(ctx) => ctx.setup(ports, certs, proxies).await,
            PortContextKind::Udp(ctx) => ctx.setup(proxies).await,
            PortContextKind::Reserved => Ok(()),
        }
    }

    pub fn apply(&mut self, new: Self) {
        match (&mut self.kind, new.kind) {
            (PortContextKind::Tcp(old), PortContextKind::Tcp(new)) => old.apply(new),
            (PortContextKind::Udp(old), PortContextKind::Udp(new)) => old.apply(new),
            (PortContextKind::Http(old), PortContextKind::Http(new)) => old.apply(new),
            (old, new) => *old = new,
        }
        self.entry = new.entry;
    }

    pub fn event(&mut self, event: PortContextEvent) {
        match &mut self.kind {
            PortContextKind::Tcp(ctx) => ctx.event(event),
            PortContextKind::Udp(ctx) => ctx.event(event),
            PortContextKind::Http(ctx) => ctx.event(event),
            PortContextKind::Reserved => (),
        }
    }

    pub fn status(&self) -> &PortStatus {
        match &self.kind {
            PortContextKind::Tcp(ctx) => ctx.status(),
            PortContextKind::Udp(ctx) => ctx.status(),
            PortContextKind::Http(ctx) => ctx.status(),
            PortContextKind::Reserved => {
                static STATUS: OnceCell<PortStatus> = OnceCell::new();
                STATUS.get_or_init(PortStatus::default)
            }
        }
    }

    pub fn reset(&mut self) {
        match &mut self.kind {
            PortContextKind::Tcp(ctx) => ctx.reset(),
            PortContextKind::Udp(ctx) => ctx.reset(),
            PortContextKind::Http(ctx) => ctx.reset(),
            PortContextKind::Reserved => (),
        }
    }
}

#[derive(Debug)]
pub enum PortContextKind {
    Tcp(TcpPortContext),
    Udp(UdpPortContext),
    Http(HttpPortContext),
    Reserved,
}
