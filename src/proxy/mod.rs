use self::{http::HttpPortContext, tcp::TcpPortContext, tls::TlsState};
use crate::{
    config::{
        port::{Port, PortEntry},
        AppConfig,
    },
    error::Error,
    keyring::Keyring,
};
use multiaddr::{Multiaddr, Protocol};
use once_cell::sync::OnceCell;
use serde_derive::Serialize;
use std::time::SystemTime;
use utoipa::ToSchema;

pub mod http;
pub mod tcp;
pub mod tls;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SocketState {
    Listening,
    PortAlreadyInUse,
    PermissionDenied,
    AddressNotAvailable,
    Error,
    #[default]
    Unknown,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
pub struct PortStatus {
    pub state: PortState,
    #[serde(serialize_with = "serialize_started_at")]
    #[schema(value_type = Option<u64>)]
    pub started_at: Option<SystemTime>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
pub struct PortState {
    pub socket: SocketState,
    pub tls: Option<TlsState>,
}

fn serialize_started_at<S>(
    started_at: &Option<SystemTime>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    if let Some(started_at) = started_at {
        let started_at = started_at
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        serializer.serialize_some(&started_at)
    } else {
        serializer.serialize_none()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

    pub async fn prepare(&mut self, config: &AppConfig) -> Result<(), Error> {
        match &mut self.kind {
            PortContextKind::Tcp(ctx) => ctx.prepare(config).await,
            PortContextKind::Http(ctx) => ctx.prepare(config).await,
            PortContextKind::Reserved => Ok(()),
        }
    }

    pub async fn setup(&mut self, keyring: &Keyring) -> Result<(), Error> {
        match &mut self.kind {
            PortContextKind::Tcp(ctx) => ctx.setup(keyring).await,
            PortContextKind::Http(ctx) => ctx.setup(keyring).await,
            PortContextKind::Reserved => Ok(()),
        }
    }

    pub async fn refresh(&mut self, certs: &Keyring) -> Result<(), Error> {
        match &mut self.kind {
            PortContextKind::Tcp(ctx) => ctx.refresh(certs).await,
            PortContextKind::Http(ctx) => ctx.refresh(certs).await,
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
