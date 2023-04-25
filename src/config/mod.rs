use serde_derive::{Deserialize, Serialize};

pub mod port;
pub mod storage;
pub mod tls;

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppConfig {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    File,
    Api,
}
