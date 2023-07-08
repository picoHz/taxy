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

    pub fn add(&mut self, entry: ProxyEntry) -> Result<(), Error> {
        if self.entries.contains_key(&entry.id) {
            Err(Error::IdAlreadyExists { id: entry.id })
        } else {
            self.entries.insert(entry.id.clone(), entry);
            Ok(())
        }
    }

    pub fn update(&mut self, entry: ProxyEntry) -> Result<bool, Error> {
        match self.entries.entry(entry.id.clone()) {
            Entry::Occupied(mut e) => {
                if e.get().proxy != entry.proxy {
                    e.insert(entry);
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Entry::Vacant(_) => Err(Error::IdNotFound { id: entry.id }),
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
