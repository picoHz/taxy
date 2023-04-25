use super::{Cert, CertInfo, SubjectName};
use std::{collections::HashMap, sync::Arc};

#[derive(Debug, Default)]
pub struct CertStore {
    certs: HashMap<String, Arc<Cert>>,
}

impl CertStore {
    pub fn new<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Arc<Cert>>,
    {
        Self {
            certs: iter
                .into_iter()
                .map(|cert| (cert.id().to_string(), cert))
                .collect(),
        }
    }

    pub fn find(&self, names: &[SubjectName]) -> Option<&Arc<Cert>> {
        let mut certs = self
            .certs
            .values()
            .filter(|cert| cert.is_valid() && names.iter().all(|name| cert.has_subject_name(name)))
            .collect::<Vec<_>>();
        certs.sort_by_key(|cert| cert.not_after);
        certs.first().copied()
    }

    pub fn add(&mut self, cert: Arc<Cert>) {
        self.certs.insert(cert.id().to_string(), cert);
    }

    pub fn delete(&mut self, id: &str) {
        self.certs.remove(id);
    }

    pub fn list(&self) -> Vec<CertInfo> {
        let mut list = self
            .certs
            .values()
            .map(|cert| cert.info())
            .collect::<Vec<_>>();
        list.sort_unstable_by_key(|cert| cert.id.clone());
        list
    }
}
