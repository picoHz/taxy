use crate::{
    config::{port::PortEntry, Source},
    proxy::PortStatus,
};
use serde_derive::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case", tag = "event")]
pub enum ServerEvent {
    PortTableUpdated {
        entries: Vec<PortEntry>,
        source: Source,
    },
    PortStatusUpdated {
        name: String,
        status: PortStatus,
    },
    Shutdown,
}
