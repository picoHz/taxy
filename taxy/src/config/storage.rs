use crate::certs::{acme::AcmeEntry, Cert};
use std::sync::Arc;
use taxy_api::{
    app::AppConfig,
    auth::{Account, LoginRequest, LoginResponse},
    error::Error,
    id::ShortId,
    port::PortEntry,
    site::ProxyEntry,
};

#[async_trait::async_trait]
pub trait Storage: Send + Sync + 'static {
    async fn save_app_config(&self, config: &AppConfig);
    async fn load_app_config(&self) -> AppConfig;
    async fn save_ports(&self, entries: &[PortEntry]);
    async fn load_ports(&self) -> Vec<PortEntry>;
    async fn load_proxies(&self) -> Vec<ProxyEntry>;
    async fn save_proxies(&self, proxies: &[ProxyEntry]);
    async fn save_cert(&self, cert: &Cert);
    async fn save_acme(&self, acme: &AcmeEntry);
    async fn delete_acme(&self, id: &ShortId);
    async fn delete_cert(&self, id: &str);
    async fn load_acmes(&self) -> Vec<AcmeEntry>;
    async fn load_certs(&self) -> Vec<Arc<Cert>>;
    async fn add_account(&self, name: &str, password: &str, totp: bool) -> Result<Account, Error>;
    async fn verify_account(&self, request: LoginRequest) -> Result<LoginResponse, Error>;
}
