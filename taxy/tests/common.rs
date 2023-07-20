use futures::Future;
use std::{collections::HashMap, sync::Arc};
use taxy::{
    certs::{acme::AcmeEntry, Cert},
    config::storage::Storage,
    server::{Server, ServerChannels},
};
use taxy_api::{app::AppConfig, error::Error, id::ShortId, port::PortEntry, site::ProxyEntry};
use tokio::sync::Mutex;

pub async fn with_server<S, F, O>(s: S, func: F) -> anyhow::Result<()>
where
    S: Storage,
    F: FnOnce(ServerChannels) -> O,
    O: Future<Output = anyhow::Result<()>> + Send + 'static,
{
    let (server, channels) = Server::new(s).await;
    let event_send = channels.event.clone();
    let task = tokio::spawn(server.start());
    func(channels).await?;
    event_send.send(taxy_api::event::ServerEvent::Shutdown)?;
    task.await??;
    Ok(())
}

#[derive(Debug, Default)]
pub struct TestStorage {
    inner: Mutex<Inner>,
}

#[derive(Debug, Default)]
struct Inner {
    pub config: AppConfig,
    pub ports: Vec<PortEntry>,
    pub proxies: Vec<ProxyEntry>,
    pub certs: HashMap<String, Arc<Cert>>,
    pub acems: HashMap<ShortId, AcmeEntry>,
    pub accounts: HashMap<String, String>,
}

impl TestStorage {
    pub fn builder() -> TestStorageBuilder {
        TestStorageBuilder::new()
    }
}

#[async_trait::async_trait]
impl Storage for TestStorage {
    async fn save_app_config(&self, config: &AppConfig) {
        self.inner.lock().await.config.clone_from(config);
    }

    async fn load_app_config(&self) -> AppConfig {
        self.inner.lock().await.config.clone()
    }

    async fn save_ports(&self, entries: &[PortEntry]) {
        self.inner.lock().await.ports = entries.to_vec();
    }

    async fn load_ports(&self) -> Vec<PortEntry> {
        self.inner.lock().await.ports.clone()
    }

    async fn load_proxies(&self) -> Vec<ProxyEntry> {
        self.inner.lock().await.proxies.clone()
    }

    async fn save_proxies(&self, proxies: &[ProxyEntry]) {
        self.inner.lock().await.proxies = proxies.to_vec();
    }

    async fn save_cert(&self, cert: &Cert) {
        self.inner
            .lock()
            .await
            .certs
            .insert(cert.id().to_string(), Arc::new(cert.clone()));
    }

    async fn save_acme(&self, acme: &AcmeEntry) {
        self.inner
            .lock()
            .await
            .acems
            .insert(*acme.id(), acme.clone());
    }

    async fn delete_acme(&self, id: &ShortId) {
        self.inner.lock().await.acems.remove(id);
    }

    async fn delete_cert(&self, id: &str) {
        self.inner.lock().await.certs.remove(id);
    }

    async fn load_acmes(&self) -> Vec<AcmeEntry> {
        self.inner.lock().await.acems.values().cloned().collect()
    }

    async fn load_certs(&self) -> Vec<Arc<Cert>> {
        self.inner.lock().await.certs.values().cloned().collect()
    }

    async fn add_account(&self, name: &str, password: &str) -> Result<(), Error> {
        self.inner
            .lock()
            .await
            .accounts
            .insert(name.to_string(), password.to_string());
        Ok(())
    }

    async fn verify_account(&self, name: &str, password: &str) -> Result<(), Error> {
        let inner = self.inner.lock().await;
        if let Some(p) = inner.accounts.get(name) {
            if p == password {
                return Ok(());
            }
        }
        Err(Error::InvalidLoginCredentials)
    }
}

#[derive(Debug, Default)]
pub struct TestStorageBuilder {
    inner: Inner,
}

impl TestStorageBuilder {
    pub fn new() -> Self {
        Self {
            inner: Inner::default(),
        }
    }

    #[allow(dead_code)]
    pub fn config(mut self, config: AppConfig) -> Self {
        self.inner.config = config;
        self
    }

    #[allow(dead_code)]
    pub fn ports(mut self, ports: Vec<PortEntry>) -> Self {
        self.inner.ports = ports;
        self
    }

    #[allow(dead_code)]
    pub fn proxies(mut self, proxies: Vec<ProxyEntry>) -> Self {
        self.inner.proxies = proxies;
        self
    }

    #[allow(dead_code)]
    pub fn certs(mut self, certs: HashMap<String, Arc<Cert>>) -> Self {
        self.inner.certs = certs;
        self
    }

    #[allow(dead_code)]
    pub fn acems(mut self, acems: HashMap<ShortId, AcmeEntry>) -> Self {
        self.inner.acems = acems;
        self
    }

    #[allow(dead_code)]
    pub fn accounts(mut self, accounts: HashMap<String, String>) -> Self {
        self.inner.accounts = accounts;
        self
    }

    pub fn build(self) -> TestStorage {
        TestStorage {
            inner: Mutex::new(self.inner),
        }
    }
}
