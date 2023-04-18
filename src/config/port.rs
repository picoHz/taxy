use multiaddr::Multiaddr;
use serde_derive::{Deserialize, Serialize};

use super::tls::TlsTermination;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackendServer {
    pub addr: Multiaddr,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortEntry {
    pub name: String,
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

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortOptions {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tls_termination: Option<TlsTermination>,
}
