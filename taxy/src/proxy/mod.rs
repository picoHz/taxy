use self::{http::HttpPortContext, tcp::TcpPortContext};
use crate::server::cert_list::CertList;
use multiaddr::{Multiaddr, Protocol};
use once_cell::sync::OnceCell;
use taxy_api::error::Error;
use taxy_api::port::{PortStatus, SocketState};
use taxy_api::{
    port::{Port, PortEntry},
    site::ProxyEntry,
};

pub mod http;
pub mod tcp;
pub mod tls;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PortContextEvent {
    SocketStateUpadted(SocketState),
}

#[derive(Debug)]
pub struct PortContext {
    pub entry: PortEntry,
    pub kind: PortContextKind,
}

impl PortContext {
    pub fn new(entry: PortEntry) -> Result<Self, Error> {
        let kind = match entry.port.listen.into_iter().last() {
            Some(Protocol::Http) | Some(Protocol::Https) => {
                PortContextKind::Http(HttpPortContext::new(&entry)?)
            }
            _ => PortContextKind::Tcp(TcpPortContext::new(&entry)?),
        };
        Ok(Self { entry, kind })
    }

    pub fn reserved() -> Self {
        Self {
            entry: PortEntry {
                id: String::new(),
                port: Port {
                    name: String::new(),
                    listen: Multiaddr::empty(),
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

    pub async fn setup(&mut self, certs: &CertList, proxies: Vec<ProxyEntry>) -> Result<(), Error> {
        match &mut self.kind {
            PortContextKind::Tcp(ctx) => ctx.setup(certs, proxies).await,
            PortContextKind::Http(ctx) => ctx.setup(certs, proxies).await,
            PortContextKind::Reserved => Ok(()),
        }
    }

    pub fn apply(&mut self, new: Self) {
        match (&mut self.kind, new.kind) {
            (PortContextKind::Tcp(old), PortContextKind::Tcp(new)) => old.apply(new),
            (PortContextKind::Http(old), PortContextKind::Http(new)) => old.apply(new),
            (old, new) => *old = new,
        }
        self.entry = new.entry;
    }

    pub fn event(&mut self, event: PortContextEvent) {
        match &mut self.kind {
            PortContextKind::Tcp(ctx) => ctx.event(event),
            PortContextKind::Http(ctx) => ctx.event(event),
            PortContextKind::Reserved => (),
        }
    }

    pub fn status(&self) -> &PortStatus {
        match &self.kind {
            PortContextKind::Tcp(ctx) => ctx.status(),
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
            PortContextKind::Http(ctx) => ctx.reset(),
            PortContextKind::Reserved => (),
        }
    }
}

#[derive(Debug)]
pub enum PortContextKind {
    Tcp(TcpPortContext),
    Http(HttpPortContext),
    Reserved,
}
