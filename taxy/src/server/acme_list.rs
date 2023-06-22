use crate::certs::acme::AcmeEntry;
use indexmap::IndexMap;
use taxy_api::error::Error;

#[derive(Debug, Default)]
pub struct AcmeList {
    entries: IndexMap<String, AcmeEntry>,
}

impl FromIterator<AcmeEntry> for AcmeList {
    fn from_iter<I: IntoIterator<Item = AcmeEntry>>(iter: I) -> Self {
        Self {
            entries: iter
                .into_iter()
                .map(|acme| (acme.id.clone(), acme))
                .collect(),
        }
    }
}

impl AcmeList {
    pub fn get(&self, id: &str) -> Option<&AcmeEntry> {
        self.entries.get(id)
    }

    pub fn entries(&self) -> impl Iterator<Item = &AcmeEntry> {
        self.entries.values()
    }

    pub fn add(&mut self, entry: AcmeEntry) -> Result<(), Error> {
        if self.entries.contains_key(&entry.id) {
            Err(Error::IdAlreadyExists { id: entry.id })
        } else {
            self.entries.insert(entry.id.clone(), entry);
            Ok(())
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
