use multiaddr::Multiaddr;
use serde_derive::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::tls::TlsTermination;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct BackendServer {
    #[schema(value_type = String, example = "/dns/example.com/tcp/8080")]
    pub addr: Multiaddr,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct PortEntry {
    pub id: String,
    #[schema(value_type = String, example = "/ip4/127.0.0.1/tcp/8080")]
    pub listen: Multiaddr,
    pub servers: Vec<BackendServer>,
    #[serde(flatten, default)]
    pub opts: PortOptions,
}

impl From<PortEntryRequest> for PortEntry {
    fn from(req: PortEntryRequest) -> Self {
        Self {
            id: cuid2::cuid(),
            listen: req.listen,
            servers: req.servers,
            opts: req.opts,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, ToSchema)]
pub struct PortEntryRequest {
    #[schema(value_type = String, example = "/ip4/127.0.0.1/tcp/8080")]
    pub listen: Multiaddr,
    pub servers: Vec<BackendServer>,
    #[serde(flatten, default)]
    pub opts: PortOptions,
}

impl From<(String, IdlessPortEntry)> for PortEntry {
    fn from((id, entry): (String, IdlessPortEntry)) -> Self {
        Self {
            id,
            listen: entry.listen,
            servers: entry.servers,
            opts: entry.opts,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdlessPortEntry {
    pub listen: Multiaddr,
    pub servers: Vec<BackendServer>,
    #[serde(flatten, default)]
    pub opts: PortOptions,
}

impl From<PortEntry> for (String, IdlessPortEntry) {
    fn from(entry: PortEntry) -> Self {
        (
            entry.id,
            IdlessPortEntry {
                listen: entry.listen,
                servers: entry.servers,
                opts: entry.opts,
            },
        )
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct PortOptions {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tls_termination: Option<TlsTermination>,
}
