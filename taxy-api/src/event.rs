use crate::acme::AcmeInfo;
use crate::app::AppConfig;
use crate::cert::CertInfo;
use crate::port::PortStatus;
use crate::{port::PortEntry, site::ProxyEntry};
use serde_derive::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case", tag = "event")]
#[non_exhaustive]
pub enum ServerEvent {
    AppConfigUpdated { config: AppConfig },
    PortTableUpdated { entries: Vec<PortEntry> },
    PortStatusUpdated { id: String, status: PortStatus },
    CertsUpdated { entries: Vec<CertInfo> },
    ProxiesUpdated { entries: Vec<ProxyEntry> },
    AcmeUpdated { entries: Vec<AcmeInfo> },
    Shutdown,
}
