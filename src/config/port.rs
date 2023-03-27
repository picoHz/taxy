use multiaddr::Multiaddr;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackendServer {
    pub addr: Multiaddr,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortEntry {
    pub name: String,
    pub listen: Multiaddr,
    pub servers: Vec<BackendServer>,
}

impl From<(String, NamelessPortEntry)> for PortEntry {
    fn from((name, entry): (String, NamelessPortEntry)) -> Self {
        Self {
            name,
            listen: entry.listen,
            servers: entry.servers,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NamelessPortEntry {
    pub listen: Multiaddr,
    pub servers: Vec<BackendServer>,
}

impl From<PortEntry> for (String, NamelessPortEntry) {
    fn from(entry: PortEntry) -> Self {
        (
            entry.name,
            NamelessPortEntry {
                listen: entry.listen,
                servers: entry.servers,
            },
        )
    }
}
