use indexmap::map::Entry;
use indexmap::IndexMap;
use taxy_api::error::Error;
use taxy_api::site::ProxyEntry;

#[derive(Debug, Default)]
pub struct ProxyList {
    entries: IndexMap<String, ProxyEntry>,
}

impl FromIterator<ProxyEntry> for ProxyList {
    fn from_iter<I: IntoIterator<Item = ProxyEntry>>(iter: I) -> Self {
        Self {
            entries: iter
                .into_iter()
                .map(|site| (site.id.clone(), site))
                .collect(),
        }
    }
}

impl ProxyList {
    pub fn get(&self, id: &str) -> Option<&ProxyEntry> {
        self.entries.get(id)
    }

    pub fn entries(&self) -> impl Iterator<Item = &ProxyEntry> {
        self.entries.values()
    }

    pub fn set(&mut self, entry: ProxyEntry) -> bool {
        match self.entries.entry(entry.id.clone()) {
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

    pub fn delete(&mut self, id: &str) -> Result<(), Error> {
        if !self.entries.contains_key(id) {
            Err(Error::IdNotFound { id: id.to_string() })
        } else {
            self.entries.remove(id);
            Ok(())
        }
    }
}
