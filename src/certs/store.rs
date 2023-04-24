use super::{Cert, CertInfo};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct CertStore {
    certs: HashMap<String, Cert>,
}

impl CertStore {
    pub fn new<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Cert>,
    {
        Self {
            certs: iter
                .into_iter()
                .map(|cert| (cert.id().to_string(), cert))
                .collect(),
        }
    }

    pub fn add(&mut self, cert: Cert) {
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
