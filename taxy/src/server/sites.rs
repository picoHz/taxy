use crate::{config::site::SiteEntry, error::Error};
use indexmap::IndexMap;

#[derive(Debug, Default)]
pub struct SiteTable {
    sites: IndexMap<String, SiteEntry>,
}

impl SiteTable {
    pub fn new(sites: Vec<SiteEntry>) -> Self {
        Self {
            sites: sites
                .into_iter()
                .map(|site| (site.id.clone(), site))
                .collect(),
        }
    }

    pub fn entries(&self) -> Vec<SiteEntry> {
        self.sites.values().cloned().collect()
    }

    pub fn add(&mut self, entry: SiteEntry) -> Result<(), Error> {
        if self.sites.contains_key(&entry.id) {
            Err(Error::IdAlreadyExists { id: entry.id })
        } else {
            self.sites.insert(entry.id.clone(), entry);
            Ok(())
        }
    }

    pub fn update(&mut self, entry: SiteEntry) -> Result<(), Error> {
        if !self.sites.contains_key(&entry.id) {
            Err(Error::IdNotFound { id: entry.id })
        } else {
            self.sites.insert(entry.id.clone(), entry);
            Ok(())
        }
    }

    pub fn delete(&mut self, id: &str) -> Result<(), Error> {
        if !self.sites.contains_key(id) {
            Err(Error::IdNotFound { id: id.to_string() })
        } else {
            self.sites.remove(id);
            Ok(())
        }
    }
}
