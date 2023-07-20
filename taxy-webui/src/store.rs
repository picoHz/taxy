use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use taxy_api::{
    acme::AcmeInfo,
    cert::CertInfo,
    id::ShortId,
    port::{PortEntry, PortStatus},
    site::ProxyEntry,
};
use yewdux::prelude::*;

#[derive(Default, Clone, PartialEq, Serialize, Deserialize, Store)]
#[store(storage = "local")]
pub struct SessionStore {
    pub token: Option<String>,
}

#[derive(Default, Clone, PartialEq, Store)]
pub struct PortStore {
    pub entries: Vec<PortEntry>,
    pub statuses: HashMap<ShortId, PortStatus>,
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
