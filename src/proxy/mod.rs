pub mod tcp;
use std::time::SystemTime;

use self::tcp::TcpPortContext;
use crate::{config::port::PortEntry, error::Error};
use serde_derive::Serialize;

const MAX_NAME_LEN: usize = 32;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize)]
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

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize)]
pub struct PortStatus {
    pub socket: SocketState,
    #[serde(serialize_with = "serialize_started_at")]
    pub started_at: Option<SystemTime>,
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
    SokcetStateUpadted(SocketState),
}

#[derive(Debug)]
pub enum PortContext {
    Tcp(TcpPortContext),
}

impl PortContext {
    pub fn new(port: &PortEntry) -> Result<Self, Error> {
        if port.name.is_empty() || port.name.len() > MAX_NAME_LEN {
            return Err(Error::InvalidName {
                name: port.name.clone(),
            });
        }
        Ok(Self::Tcp(TcpPortContext::new(port)?))
    }

    pub fn apply(&mut self, new: Self) {
        match (self, new) {
            (Self::Tcp(old), Self::Tcp(new)) => old.apply(new),
        }
    }

    pub fn event(&mut self, event: PortContextEvent) {
        match self {
            Self::Tcp(ctx) => ctx.event(event),
        }
    }

    pub fn status(&self) -> &PortStatus {
        match self {
            Self::Tcp(ctx) => ctx.status(),
        }
    }
}
