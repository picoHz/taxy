use crate::{
    config::{port::PortEntry, AppConfig, Source},
    keyring::KeyringInfo,
    proxy::PortStatus,
};
use serde_derive::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "snake_case", tag = "event")]
pub enum ServerEvent {
    AppConfigUpdated {
        config: AppConfig,
        source: Source,
    },
    PortTableUpdated {
        entries: Vec<PortEntry>,
        source: Source,
    },
    PortStatusUpdated {
        id: String,
        status: PortStatus,
    },
    KeyringUpdated {
        items: Vec<KeyringInfo>,
    },
    Shutdown,
}
