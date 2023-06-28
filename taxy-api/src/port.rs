use crate::tls::{TlsState, TlsTermination};
use multiaddr::Multiaddr;
use serde_derive::{Deserialize, Serialize};
use std::{net::IpAddr, time::SystemTime};
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Port {
    #[schema(value_type = String, example = "/ip4/127.0.0.1/tcp/8080")]
    pub listen: Multiaddr,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct NetworkInterface {
    pub name: String,
    pub description: String,
    pub addrs: Vec<NetworkAddr>,
    pub mac: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct NetworkAddr {
    #[schema(value_type = String, example = "127.0.0.1")]
    pub ip: IpAddr,
    #[schema(value_type = String, example = "255.255.255.0")]
    pub mask: IpAddr,
}
