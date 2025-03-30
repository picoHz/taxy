use crate::certs::acme::AcmeEntry;
use indexmap::IndexMap;
use taxy_api::{acme::AcmeConfig, error::Error, id::ShortId};

#[derive(Debug, Default)]
pub struct AcmeList {
    entries: IndexMap<ShortId, AcmeEntry>,
}

impl FromIterator<AcmeEntry> for AcmeList {
    fn from_iter<I: IntoIterator<Item = AcmeEntry>>(iter: I) -> Self {
        Self {
            entries: iter.into_iter().map(|acme| (acme.id, acme)).collect(),
        }
    }
}

impl AcmeList {
    pub fn get(&self, id: ShortId) -> Option<&AcmeEntry> {
        self.entries.get(&id)
    }

    pub fn entries(&self) -> impl Iterator<Item = &AcmeEntry> {
        self.entries.values()
    }

    pub fn add(&mut self, entry: AcmeEntry) -> Result<(), Error> {
        if self.entries.contains_key(&entry.id) {
            Err(Error::IdAlreadyExists { id: entry.id })
        } else {
            self.entries.insert(entry.id, entry);
            Ok(())
        }
    }

    pub fn update(&mut self, id: ShortId, config: AcmeConfig) -> Result<AcmeEntry, Error> {
        if let Some(entry) = self.entries.get_mut(&id) {
            entry.acme.config = config;
            Ok(entry.clone())
        } else {
            Err(Error::IdNotFound { id: id.to_string() })
        }
    }

    pub fn delete(&mut self, id: ShortId) -> Result<(), Error> {
        if !self.entries.contains_key(&id) {
            Err(Error::IdNotFound { id: id.to_string() })
        } else {
            self.entries.swap_remove(&id);
            Ok(())
        }
    }
}
