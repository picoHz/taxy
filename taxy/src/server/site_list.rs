use indexmap::map::Entry;
use indexmap::IndexMap;
use taxy_api::error::Error;
use taxy_api::site::SiteEntry;

#[derive(Debug, Default)]
pub struct SiteList {
    entries: IndexMap<String, SiteEntry>,
}

impl FromIterator<SiteEntry> for SiteList {
    fn from_iter<I: IntoIterator<Item = SiteEntry>>(iter: I) -> Self {
        Self {
            entries: iter
                .into_iter()
                .map(|site| (site.id.clone(), site))
                .collect(),
        }
    }
}

impl SiteList {
    pub fn get(&self, id: &str) -> Option<&SiteEntry> {
        self.entries.get(id)
    }

    pub fn entries(&self) -> impl Iterator<Item = &SiteEntry> {
        self.entries.values()
    }

    pub fn add(&mut self, entry: SiteEntry) -> Result<(), Error> {
        if self.entries.contains_key(&entry.id) {
            Err(Error::IdAlreadyExists { id: entry.id })
        } else {
            self.entries.insert(entry.id.clone(), entry);
            Ok(())
        }
    }

    pub fn update(&mut self, entry: SiteEntry) -> Result<bool, Error> {
        match self.entries.entry(entry.id.clone()) {
            Entry::Occupied(mut e) => {
                if e.get().site != entry.site {
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
