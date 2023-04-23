use crate::{
    config::{port::PortEntry, AppConfig, Source},
    proxy::PortStatus, certs::CertInfo,
};
use serde_derive::Serialize;

#[derive(Debug, Clone, Serialize)]
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
        name: String,
        status: PortStatus,
    },
    CertListUpdated {
        certs: Vec<CertInfo>
    },
    Shutdown,
}
