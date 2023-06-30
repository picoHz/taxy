use crate::certs::Cert;
use indexmap::IndexMap;
use log::warn;
use std::sync::Arc;
use taxy_api::{cert::CertKind, error::Error};
use tokio_rustls::rustls::{Certificate, RootCertStore};

#[derive(Debug)]
pub struct CertList {
    certs: IndexMap<String, Arc<Cert>>,
    system_root_certs: RootCertStore,
    root_certs: RootCertStore,
}

impl CertList {
    pub async fn new<I: IntoIterator<Item = Arc<Cert>>>(iter: I) -> Self {
        let mut certs = iter
            .into_iter()
            .map(|cert| (cert.id().to_string(), cert))
            .collect::<IndexMap<_, _>>();
        certs.sort_unstable_by(|_, v1, _, v2| v1.cmp(v2));

        let mut system_root_certs = RootCertStore::empty();
        if let Ok(certs) = tokio::task::spawn_blocking(rustls_native_certs::load_native_certs).await
        {
            match certs {
                Ok(certs) => {
                    for certs in certs {
                        if let Err(err) = system_root_certs.add(&Certificate(certs.0)) {
                            warn!("failed to add native certs: {err}");
                        }
                    }
                }
                Err(err) => {
                    warn!("failed to load native certs: {err}");
                }
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

    pub fn add(&mut self, cert: Arc<Cert>) -> Result<(), Error> {
        if self.certs.contains_key(cert.id()) {
            Err(Error::IdAlreadyExists {
                id: cert.id().to_string(),
            })
        } else {
            self.certs.insert(cert.id().to_string(), cert.clone());
            self.certs.sort_unstable_by(|_, v1, _, v2| v1.cmp(v2));
            if cert.kind == CertKind::Root {
                self.update_root_certs();
            }
            Ok(())
        }
    }

    pub fn delete(&mut self, id: &str) -> Result<(), Error> {
        if !self.certs.contains_key(id) {
            Err(Error::IdNotFound { id: id.to_string() })
        } else {
            if let Some(cert) = self.certs.remove(id) {
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
                if let Ok(certified) = cert.certified() {
                    for cert in certified.cert {
                        if let Err(err) = root_certs.add(&cert) {
                            warn!("failed to add root cert: {}", err);
                        }
                    }
                }
            }
        }
        self.root_certs = root_certs;
    }
}
