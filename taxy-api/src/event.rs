use crate::acme::AcmeInfo;
use crate::app::AppConfig;
use crate::cert::CertInfo;
use crate::id::ShortId;
use crate::port::PortStatus;
use crate::proxy::ProxyStatus;
use crate::{port::PortEntry, proxy::ProxyEntry};
use serde_derive::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case", tag = "event")]
#[non_exhaustive]
pub enum ServerEvent {
    AppConfigUpdated { config: AppConfig },
    PortTableUpdated { entries: Vec<PortEntry> },
    PortStatusUpdated { id: ShortId, status: PortStatus },
    CertsUpdated { entries: Vec<CertInfo> },
    ProxiesUpdated { entries: Vec<ProxyEntry> },
    ProxyStatusUpdated { id: ShortId, status: ProxyStatus },
    AcmeUpdated { entries: Vec<AcmeInfo> },
    Shutdown,
}
