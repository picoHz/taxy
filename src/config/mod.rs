use self::certs::CertsConfig;
use serde_derive::{Deserialize, Serialize};

pub mod certs;
pub mod port;
pub mod storage;

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppConfig {
    pub certs: CertsConfig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    File,
    Api,
}
