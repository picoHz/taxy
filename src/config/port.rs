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
    pub name: String,
    #[schema(value_type = String, example = "/ip4/127.0.0.1/tcp/8080")]
    pub listen: Multiaddr,
    pub servers: Vec<BackendServer>,
    #[serde(flatten, default)]
    pub opts: PortOptions,
}

impl From<(String, NamelessPortEntry)> for PortEntry {
    fn from((name, entry): (String, NamelessPortEntry)) -> Self {
        Self {
            name,
            listen: entry.listen,
            servers: entry.servers,
            opts: entry.opts,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NamelessPortEntry {
    pub listen: Multiaddr,
    pub servers: Vec<BackendServer>,
    #[serde(flatten, default)]
    pub opts: PortOptions,
}

impl From<PortEntry> for (String, NamelessPortEntry) {
    fn from(entry: PortEntry) -> Self {
        (
            entry.name,
            NamelessPortEntry {
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
