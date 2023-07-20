use indexmap::map::Entry;
use indexmap::IndexMap;
use multiaddr::Protocol;
use taxy_api::error::Error;
use taxy_api::id::ShortId;
use taxy_api::port::PortEntry;
use taxy_api::site::{Proxy, ProxyEntry, ProxyKind};

#[derive(Debug, Default)]
pub struct ProxyList {
    entries: IndexMap<ShortId, ProxyEntry>,
}

impl FromIterator<ProxyEntry> for ProxyList {
    fn from_iter<I: IntoIterator<Item = ProxyEntry>>(iter: I) -> Self {
        Self {
            entries: iter.into_iter().map(|site| (site.id, site)).collect(),
        }
    }
}

impl ProxyList {
    pub fn get(&self, id: &ShortId) -> Option<&ProxyEntry> {
        self.entries.get(id)
    }

    pub fn entries(&self) -> impl Iterator<Item = &ProxyEntry> {
        self.entries.values()
    }

    pub fn set(&mut self, entry: ProxyEntry) -> bool {
        self.remove_deplicate_ports(&entry.proxy);
        match self.entries.entry(entry.id) {
            Entry::Occupied(mut e) => {
                if e.get().proxy != entry.proxy {
                    e.insert(entry);
                    true
                } else {
                    false
                }
            }
            Entry::Vacant(inner) => {
                inner.insert(entry);
                true
            }
        }
    }

    pub fn delete(&mut self, id: &ShortId) -> Result<(), Error> {
        if !self.entries.contains_key(id) {
            Err(Error::IdNotFound { id: id.to_string() })
        } else {
            self.entries.remove(id);
            Ok(())
        }
    }

    pub fn remove_incompatible_ports(&mut self, ports: &[PortEntry]) -> bool {
        let mut changed = false;
        for entry in self.entries.values_mut() {
            let len = entry.proxy.ports.len();
            entry.proxy.ports = entry
                .proxy
                .ports
                .drain(..)
                .filter(|port| {
                    ports
                        .iter()
                        .find(|p| p.id == *port)
                        .map(|port| {
                            port.port
                                .listen
                                .iter()
                                .any(|item| matches!(item, Protocol::Http | Protocol::Https))
                                ^ (matches!(entry.proxy.kind, ProxyKind::Tcp(_)))
                        })
                        .unwrap_or_default()
                })
                .collect();
            changed |= len != entry.proxy.ports.len();
        }
        changed
    }

    fn remove_deplicate_ports(&mut self, proxy: &Proxy) {
        if let ProxyKind::Tcp(_) = &proxy.kind {
            for entry in self.entries.values_mut() {
                entry.proxy.ports = entry
                    .proxy
                    .ports
                    .drain(..)
                    .filter(|port| !proxy.ports.contains(port))
                    .collect();
            }
        }
    }
}
