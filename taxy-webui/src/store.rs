use serde_derive::{Deserialize, Serialize};
use yewdux::prelude::*;

#[derive(Default, Clone, PartialEq, Serialize, Deserialize, Store)]
#[store(storage = "local")]
pub struct UserSession {
    pub token: Option<String>,
}
