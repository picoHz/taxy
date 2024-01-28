use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use taxy_api::{
    acme::AcmeInfo,
    cert::CertInfo,
    id::ShortId,
    port::{PortEntry, PortStatus},
    proxy::{ProxyEntry, ProxyStatus},
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
    pub loaded: bool,
}

#[derive(Default, Clone, PartialEq, Store)]
pub struct ProxyStore {
    pub entries: Vec<ProxyEntry>,
    pub statuses: HashMap<ShortId, ProxyStatus>,
    pub loaded: bool,
}

#[derive(Default, Clone, PartialEq, Store)]
pub struct CertStore {
    pub entries: Vec<CertInfo>,
    pub loaded: bool,
}

#[derive(Default, Clone, PartialEq, Store)]
pub struct AcmeStore {
    pub entries: Vec<AcmeInfo>,
    pub loaded: bool,
}
