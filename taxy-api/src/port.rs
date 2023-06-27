use crate::{
    error::Error,
    tls::{TlsState, TlsTermination},
};
use multiaddr::Multiaddr;
use serde_derive::{Deserialize, Serialize};
use std::{net::SocketAddr, str::FromStr, time::SystemTime};
use utoipa::ToSchema;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct PortStatus {
    pub state: PortState,
    #[serde(serialize_with = "serialize_started_at")]
    #[schema(value_type = Option<u64>)]
    pub started_at: Option<SystemTime>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct UpstreamServer {
    #[schema(value_type = String, example = "/dns/example.com/tcp/8080")]
    pub addr: Multiaddr,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct PortEntry {
    pub id: String,
    #[schema(inline)]
    #[serde(flatten)]
    pub port: Port,
}

impl From<(String, Port)> for PortEntry {
    fn from((id, port): (String, Port)) -> Self {
        Self { id, port }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Protocol {
    Tcp,
    Tls,
    Http,
    Https,
}

impl Protocol {
    pub fn is_tls(&self) -> bool {
        matches!(self, Self::Tls | Self::Https)
    }

    pub fn is_http(&self) -> bool {
        matches!(self, Self::Http | Self::Https)
    }
}

impl ToString for Protocol {
    fn to_string(&self) -> String {
        match self {
            Self::Tcp => "tcp".to_owned(),
            Self::Tls => "tls".to_owned(),
            Self::Http => "http".to_owned(),
            Self::Https => "https".to_owned(),
        }
    }
}

impl FromStr for Protocol {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "tcp" => Ok(Self::Tcp),
            "tls" => Ok(Self::Tls),
            "http" => Ok(Self::Http),
            "https" => Ok(Self::Https),
            _ => Err(Error::InvalidProtocol { name: s.to_owned() }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Port {
    pub protocol: Protocol,
    #[schema(value_type = [String], example = json!(["127.0.0.1:8080"]))]
    pub bind: Vec<SocketAddr>,
    #[serde(flatten, default)]
    pub opts: PortOptions,
}

impl From<PortEntry> for (String, Port) {
    fn from(entry: PortEntry) -> Self {
        (entry.id, entry.port)
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct PortOptions {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub upstream_servers: Vec<UpstreamServer>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tls_termination: Option<TlsTermination>,
}
