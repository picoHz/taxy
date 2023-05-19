use self::{
    acme::{AcmeEntry, AcmeInfo},
    certs::{Cert, CertInfo},
};
use base64::{engine::general_purpose, Engine as _};
use log::info;
use once_cell::sync::OnceCell;
use rand::RngCore;
use serde_derive::Serialize;
use std::{collections::HashMap, sync::Arc};
use utoipa::ToSchema;
use zeroize::Zeroizing;

pub mod acme;
pub mod certs;
pub mod subject_name;

use subject_name::SubjectName;

#[derive(Debug, Default)]
pub struct Keyring {
    certs: HashMap<String, KeyringItem>,
}

#[derive(Debug)]
pub enum KeyringItem {
    ServerCert(Arc<Cert>),
    Acme(Arc<AcmeEntry>),
}

impl KeyringItem {
    pub fn id(&self) -> &str {
        match self {
            Self::ServerCert(cert) => cert.id(),
            Self::Acme(acme) => acme.id(),
        }
    }

    pub fn info(&self) -> KeyringInfo {
        match self {
            Self::ServerCert(cert) => KeyringInfo::ServerCert(cert.info()),
            Self::Acme(acme) => KeyringInfo::Acme(acme.info()),
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum KeyringInfo {
    ServerCert(CertInfo),
    Acme(AcmeInfo),
}

impl KeyringInfo {
    pub fn id(&self) -> &str {
        match self {
            Self::ServerCert(cert) => &cert.id,
            Self::Acme(acme) => &acme.id,
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

    pub fn iter(&self) -> impl Iterator<Item = &KeyringItem> {
        self.certs.values()
    }

    pub fn certs(&self) -> Vec<Arc<Cert>> {
        let mut certs = self
            .certs
            .values()
            .filter_map(|item| match item {
                KeyringItem::ServerCert(cert) => Some(cert.clone()),
                _ => None,
            })
            .collect::<Vec<_>>();
        certs.sort();
        certs
    }

    pub fn find_server_cert_by_acme(&self, acme: &str) -> Vec<&Arc<Cert>> {
        self.certs
            .values()
            .filter_map(|item| match item {
                KeyringItem::ServerCert(cert) => Some(cert),
                _ => None,
            })
            .filter(|cert| {
                cert.metadata
                    .as_ref()
                    .map_or(false, |meta| meta.acme_id == acme)
            })
            .collect()
    }

    pub fn add(&mut self, item: KeyringItem) {
        self.certs.insert(item.id().to_string(), item);
    }

    pub fn delete(&mut self, id: &str) -> Option<KeyringItem> {
        self.certs.remove(id)
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

const APPKEY_KEYRING_SERVICE: &str = "taxy.encryption";
const APPKEY_KEYRING_USER: &str = "taxy";
const APPKEY_LENGTH: usize = 32;

pub fn load_appkey() -> anyhow::Result<Zeroizing<Vec<u8>>> {
    static ENTRY: OnceCell<keyring::Entry> = OnceCell::new();
    let entry = ENTRY
        .get_or_try_init(|| keyring::Entry::new(APPKEY_KEYRING_SERVICE, APPKEY_KEYRING_USER))?;

    let password: keyring::Result<Zeroizing<String>> = entry.get_password().map(Zeroizing::new);
    match password {
        Ok(password) => Ok(Zeroizing::new(
            general_purpose::STANDARD_NO_PAD.decode::<&str>(&password)?,
        )),
        Err(keyring::Error::NoEntry) => {
            info!("generating appkey...");
            let mut key = Zeroizing::new(vec![0u8; APPKEY_LENGTH]);
            rand::thread_rng().fill_bytes(key.as_mut());
            let password = Zeroizing::new(general_purpose::STANDARD_NO_PAD.encode(&key));
            entry.set_password(&password)?;
            Ok(key)
        }
        Err(err) => Err(err.into()),
    }
}
