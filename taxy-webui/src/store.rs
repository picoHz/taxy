use serde_derive::{Deserialize, Serialize};
use taxy_api::{acme::AcmeInfo, cert::CertInfo, port::PortEntry, site::ProxyEntry};
use yewdux::prelude::*;

#[derive(Default, Clone, PartialEq, Serialize, Deserialize, Store)]
#[store(storage = "local")]
pub struct SessionStore {
    pub token: Option<String>,
}

#[derive(Default, Clone, PartialEq, Store)]
pub struct PortStore {
    pub entries: Vec<PortEntry>,
}

#[derive(Default, Clone, PartialEq, Store)]
pub struct ProxyStore {
    pub entries: Vec<ProxyEntry>,
}

#[derive(Default, Clone, PartialEq, Store)]
pub struct CertStore {
    pub entries: Vec<CertInfo>,
}

#[derive(Default, Clone, PartialEq, Store)]
pub struct AcmeStore {
    pub entries: Vec<AcmeInfo>,
}
