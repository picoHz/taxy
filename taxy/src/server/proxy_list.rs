use indexmap::map::Entry;
use indexmap::IndexMap;
use taxy_api::error::Error;
use taxy_api::id::ShortId;
use taxy_api::port::PortEntry;
use taxy_api::proxy::{Proxy, ProxyEntry, ProxyKind, ProxyState, ProxyStatus};

#[derive(Debug)]
pub struct ProxyContext {
    pub entry: ProxyEntry,
    pub status: ProxyStatus,
}

impl ProxyContext {
    fn new(entry: ProxyEntry) -> Self {
        let state = if entry.proxy.active && !entry.proxy.ports.is_empty() {
            ProxyState::Active
        } else {
            ProxyState::Inactive
        };
        Self {
            entry,
            status: ProxyStatus { state },
        }
    }
}

#[derive(Debug, Default)]
pub struct ProxyList {
    entries: IndexMap<ShortId, ProxyContext>,
}

impl FromIterator<ProxyEntry> for ProxyList {
    fn from_iter<I: IntoIterator<Item = ProxyEntry>>(iter: I) -> Self {
        Self {
            entries: iter
                .into_iter()
                .map(|proxy| (proxy.id, ProxyContext::new(proxy)))
                .collect(),
        }
    }
}

impl ProxyList {
    pub fn get(&self, id: ShortId) -> Option<&ProxyContext> {
        self.entries.get(&id)
    }

    pub fn entries(&self) -> impl Iterator<Item = &ProxyEntry> {
        self.entries.values().map(|ctx| &ctx.entry)
    }

    pub fn contexts(&self) -> impl Iterator<Item = &ProxyContext> {
        self.entries.values()
    }

    pub fn set(&mut self, entry: ProxyEntry) -> bool {
        self.remove_deplicate_ports(&entry.proxy);
        match self.entries.entry(entry.id) {
            Entry::Occupied(mut e) => {
                if e.get().entry.proxy != entry.proxy {
                    e.insert(ProxyContext::new(entry));
                    true
                } else {
                    false
                }
            }
            Entry::Vacant(inner) => {
                inner.insert(ProxyContext::new(entry));
                true
            }
        }
    }

    pub fn delete(&mut self, id: ShortId) -> Result<(), Error> {
        if !self.entries.contains_key(&id) {
            Err(Error::IdNotFound { id: id.to_string() })
        } else {
            self.entries.remove(&id);
            Ok(())
        }
    }

    pub fn remove_incompatible_ports(&mut self, ports: &[PortEntry]) -> bool {
        let mut changed = false;
        for ctx in self.entries.values_mut() {
            let len = ctx.entry.proxy.ports.len();
            ctx.entry.proxy.ports = ctx
                .entry
                .proxy
                .ports
                .drain(..)
                .filter(|port| {
                    ports
                        .iter()
                        .find(|p| p.id == *port)
                        .map(|port| match ctx.entry.proxy.kind {
                            ProxyKind::Http(_) => port.port.listen.is_http(),
                            ProxyKind::Tcp(_) => {
                                !port.port.listen.is_udp() && !port.port.listen.is_http()
                            }
                            ProxyKind::Udp(_) => port.port.listen.is_udp(),
                        })
                        .unwrap_or_default()
                })
                .collect();
            changed |= len != ctx.entry.proxy.ports.len();
        }
        changed
    }

    fn remove_deplicate_ports(&mut self, proxy: &Proxy) {
        if let ProxyKind::Tcp(_) = &proxy.kind {
            for ctx in self.entries.values_mut() {
                ctx.entry.proxy.ports = ctx
                    .entry
                    .proxy
                    .ports
                    .drain(..)
                    .filter(|port| !proxy.ports.contains(port))
                    .collect();
            }
        }
    }
}
