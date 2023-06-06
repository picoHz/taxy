use serde_derive::{Deserialize, Serialize};
use taxy_api::port::PortEntry;
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
