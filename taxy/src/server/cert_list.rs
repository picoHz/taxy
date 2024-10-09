use crate::certs::Cert;
use indexmap::IndexMap;
use log::warn;
use std::sync::Arc;
use taxy_api::{cert::CertKind, error::Error, id::ShortId};
use tokio_rustls::rustls::RootCertStore;

#[derive(Debug)]
pub struct CertList {
    certs: IndexMap<ShortId, Arc<Cert>>,
    system_root_certs: RootCertStore,
    root_certs: RootCertStore,
}

impl CertList {
    pub async fn new<I: IntoIterator<Item = Arc<Cert>>>(iter: I) -> Self {
        let mut certs = iter
            .into_iter()
            .map(|cert| (cert.id(), cert))
            .collect::<IndexMap<_, _>>();
        certs.sort_unstable_by(|_, v1, _, v2| v1.partial_cmp(v2).unwrap());

        let mut system_root_certs = RootCertStore::empty();
        if let Ok(result) =
            tokio::task::spawn_blocking(rustls_native_certs::load_native_certs).await
        {
            for cert in result.certs {
                if let Err(err) = system_root_certs.add(cert) {
                    warn!("failed to add native certs: {err}");
                }
            }
            for error in result.errors {
                warn!("failed to load native certs: {error}");
            }
        }

        let mut this = Self {
            certs,
            system_root_certs: system_root_certs.clone(),
            root_certs: RootCertStore::empty(),
        };
        this.update_root_certs();
        this
    }

    pub fn iter(&self) -> impl Iterator<Item = &Arc<Cert>> {
        self.certs.values()
    }

    pub fn root_certs(&self) -> &RootCertStore {
        &self.root_certs
    }

    pub fn find_certs_by_acme(&self, acme: ShortId) -> Vec<&Arc<Cert>> {
        self.certs
            .values()
            .filter(|cert| {
                cert.metadata
                    .as_ref()
                    .map_or(false, |meta| meta.acme_id == acme)
            })
            .collect()
    }

    pub fn get(&self, id: ShortId) -> Option<&Arc<Cert>> {
        self.certs.get(&id)
    }

    pub fn add(&mut self, cert: Arc<Cert>) {
        self.certs.insert(cert.id(), cert.clone());
        self.certs
            .sort_unstable_by(|_, v1, _, v2| v1.partial_cmp(v2).unwrap());
        if cert.kind == CertKind::Root {
            self.update_root_certs();
        }
    }

    pub fn delete(&mut self, id: ShortId) -> Result<(), Error> {
        if !self.certs.contains_key(&id) {
            Err(Error::IdNotFound { id: id.to_string() })
        } else {
            if let Some(cert) = self.certs.remove(&id) {
                if cert.kind == CertKind::Root {
                    self.update_root_certs();
                }
            }
            Ok(())
        }
    }

    fn update_root_certs(&mut self) {
        let mut root_certs = self.system_root_certs.clone();
        for cert in self.certs.values() {
            if cert.kind == CertKind::Root {
                if let Ok(certs) = cert.certificates() {
                    for cert in certs {
                        if let Err(err) = root_certs.add(cert) {
                            warn!("failed to add root cert: {}", err);
                        }
                    }
                }
            }
        }
        self.root_certs = root_certs;
    }
}
