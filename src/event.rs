use crate::{
    config::{port::PortEntry, site::SiteEntry, AppConfig, Source},
    keyring::{acme::AcmeInfo, certs::CertInfo},
    proxy::PortStatus,
};
use serde_derive::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "snake_case", tag = "event")]
pub enum ServerEvent {
    AppConfigUpdated { config: AppConfig, source: Source },
    PortTableUpdated { entries: Vec<PortEntry> },
    PortStatusUpdated { id: String, status: PortStatus },
    ServerCertsUpdated { items: Vec<CertInfo> },
    SitesUpdated { items: Vec<SiteEntry> },
    AcmeUpdated { items: Vec<AcmeInfo> },
    Shutdown,
}
