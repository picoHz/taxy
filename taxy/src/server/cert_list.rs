use crate::certs::Cert;
use indexmap::IndexMap;
use std::sync::Arc;
use taxy_api::error::Error;

#[derive(Debug, Default)]
pub struct CertList {
    certs: IndexMap<String, Arc<Cert>>,
}

impl FromIterator<Arc<Cert>> for CertList {
    fn from_iter<I: IntoIterator<Item = Arc<Cert>>>(iter: I) -> Self {
        let mut certs = iter
            .into_iter()
            .map(|cert| (cert.id().to_string(), cert))
            .collect::<IndexMap<_, _>>();
        certs.sort_unstable_by(|_, v1, _, v2| v1.cmp(v2));
        Self { certs }
    }
}

impl CertList {
    pub fn iter(&self) -> impl Iterator<Item = &Arc<Cert>> {
        self.certs.values()
    }

    pub fn find_certs_by_acme(&self, acme: &str) -> Vec<&Arc<Cert>> {
        self.certs
            .values()
            .filter(|cert| {
                cert.metadata
                    .as_ref()
                    .map_or(false, |meta| meta.acme_id == acme)
            })
            .collect()
    }

    pub fn get(&self, id: &str) -> Option<&Arc<Cert>> {
        self.certs.get(id)
    }

    pub fn add(&mut self, item: Arc<Cert>) -> Result<(), Error> {
        if self.certs.contains_key(item.id()) {
            Err(Error::IdAlreadyExists {
                id: item.id().to_string(),
            })
        } else {
            self.certs.insert(item.id().to_string(), item);
            self.certs.sort_unstable_by(|_, v1, _, v2| v1.cmp(v2));
            Ok(())
        }
    }

    pub fn delete(&mut self, id: &str) -> Result<(), Error> {
        if !self.certs.contains_key(id) {
            Err(Error::IdNotFound { id: id.to_string() })
        } else {
            self.certs.remove(id);
            Ok(())
        }
    }
}
