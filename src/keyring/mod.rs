use self::certs::{Cert, CertInfo};
use serde_derive::Serialize;
use std::{collections::HashMap, sync::Arc};
use utoipa::ToSchema;

pub mod certs;
pub mod store;
pub mod subject_name;

use subject_name::SubjectName;

#[derive(Debug, Default)]
pub struct Keyring {
    certs: HashMap<String, KeyringItem>,
}

#[derive(Debug)]
pub enum KeyringItem {
    ServerCert(Arc<Cert>),
}

impl KeyringItem {
    pub fn id(&self) -> &str {
        match self {
            Self::ServerCert(cert) => cert.id(),
        }
    }

    pub fn info(&self) -> KeyringInfo {
        match self {
            Self::ServerCert(cert) => KeyringInfo::ServerCert(cert.info()),
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum KeyringInfo {
    ServerCert(CertInfo),
}

impl KeyringInfo {
    pub fn id(&self) -> &str {
        match self {
            Self::ServerCert(cert) => &cert.id,
        }
    }
}

impl Keyring {
    pub fn new<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = KeyringItem>,
    {
        Self {
            certs: iter
                .into_iter()
                .map(|cert| (cert.id().to_string(), cert))
                .collect(),
        }
    }

    pub fn find_server_cert(&self, names: &[SubjectName]) -> Option<&Arc<Cert>> {
        let mut certs = self
            .certs
            .values()
            .map(|item| match item {
                KeyringItem::ServerCert(cert) => cert,
            })
            .filter(|cert| cert.is_valid() && names.iter().all(|name| cert.has_subject_name(name)))
            .collect::<Vec<_>>();
        certs.sort_by_key(|cert| cert.not_after);
        certs.first().copied()
    }

    pub fn add(&mut self, item: KeyringItem) {
        self.certs.insert(item.id().to_string(), item);
    }

    pub fn delete(&mut self, id: &str) {
        self.certs.remove(id);
    }

    pub fn list(&self) -> Vec<KeyringInfo> {
        let mut list = self
            .certs
            .values()
            .map(|cert| cert.info())
            .collect::<Vec<_>>();
        list.sort_unstable_by_key(|cert| cert.id().to_string());
        list
    }
}
