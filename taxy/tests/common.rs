#![allow(dead_code)]

use futures::Future;
use net2::TcpBuilder;
use std::{
    collections::HashMap,
    net::{SocketAddr, ToSocketAddrs},
    path::Path,
    sync::Arc,
};
use taxy::{
    certs::{acme::AcmeEntry, Cert},
    config::{new_appinfo, storage::Storage},
    server::{Server, ServerChannels},
};
use taxy_api::{
    app::AppConfig,
    auth::{Account, LoginMethod, LoginRequest, LoginResponse},
    error::Error,
    id::ShortId,
    multiaddr::Multiaddr,
    port::PortEntry,
    proxy::ProxyEntry,
};
use tokio::sync::Mutex;
use url::Url;

pub async fn with_server<S, F, O>(s: S, func: F) -> anyhow::Result<()>
where
    S: Storage,
    F: FnOnce(ServerChannels) -> O,
    O: Future<Output = anyhow::Result<()>> + Send + 'static,
{
    let app_info = new_appinfo(Path::new("."), Path::new("."));
    let (server, channels) = Server::new(app_info, s).await;
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
    pub certs: HashMap<ShortId, Arc<Cert>>,
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
            .insert(cert.id(), Arc::new(cert.clone()));
    }

    async fn save_acme(&self, acme: &AcmeEntry) {
        self.inner
            .lock()
            .await
            .acems
            .insert(acme.id(), acme.clone());
    }

    async fn delete_acme(&self, id: ShortId) {
        self.inner.lock().await.acems.remove(&id);
    }

    async fn delete_cert(&self, id: ShortId) {
        self.inner.lock().await.certs.remove(&id);
    }

    async fn load_acmes(&self) -> Vec<AcmeEntry> {
        self.inner.lock().await.acems.values().cloned().collect()
    }

    async fn load_certs(&self) -> Vec<Arc<Cert>> {
        self.inner.lock().await.certs.values().cloned().collect()
    }

    async fn add_account(&self, name: &str, password: &str, _totp: bool) -> Result<Account, Error> {
        self.inner
            .lock()
            .await
            .accounts
            .insert(name.to_string(), password.to_string());
        Ok(Account {
            password: password.to_string(),
            totp: None,
        })
    }

    async fn verify_account(&self, request: LoginRequest) -> Result<LoginResponse, Error> {
        let password = match request.method {
            LoginMethod::Password { password } => password,
            _ => return Err(Error::InvalidLoginCredentials),
        };
        let inner = self.inner.lock().await;
        if let Some(p) = inner.accounts.get(&request.username) {
            if *p == password {
                return Ok(LoginResponse::Success);
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

    pub fn config(mut self, config: AppConfig) -> Self {
        self.inner.config = config;
        self
    }

    pub fn ports(mut self, ports: Vec<PortEntry>) -> Self {
        self.inner.ports = ports;
        self
    }

    pub fn proxies(mut self, proxies: Vec<ProxyEntry>) -> Self {
        self.inner.proxies = proxies;
        self
    }

    pub fn certs(mut self, certs: HashMap<ShortId, Arc<Cert>>) -> Self {
        self.inner.certs = certs;
        self
    }

    pub fn acems(mut self, acems: HashMap<ShortId, AcmeEntry>) -> Self {
        self.inner.acems = acems;
        self
    }

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

pub fn alloc_port() -> Result<TestPort, std::io::Error> {
    let addr = "localhost:0".to_socket_addrs().unwrap().next().unwrap();
    let addr = if addr.is_ipv4() {
        TcpBuilder::new_v4()?
    } else {
        TcpBuilder::new_v6()?
    }
    .reuse_address(true)?
    .bind(addr)?
    .local_addr()?;
    Ok(TestPort { addr })
}

pub struct TestPort {
    addr: SocketAddr,
}

impl TestPort {
    pub fn socket_addr(&self) -> SocketAddr {
        self.addr
    }

    pub fn multiaddr_http(&self) -> Multiaddr {
        let protocol = if self.addr.is_ipv4() { "ip4" } else { "ip6" };
        let addr = self.addr.ip();
        format!("/{protocol}/{addr}/tcp/{}/http", self.addr.port())
            .parse()
            .unwrap()
    }

    pub fn multiaddr_https(&self) -> Multiaddr {
        let protocol = if self.addr.is_ipv4() { "ip4" } else { "ip6" };
        let addr = self.addr.ip();
        format!("/{protocol}/{addr}/tcp/{}/https", self.addr.port())
            .parse()
            .unwrap()
    }

    pub fn multiaddr_tcp(&self) -> Multiaddr {
        let protocol = if self.addr.is_ipv4() { "ip4" } else { "ip6" };
        let addr = self.addr.ip();
        format!("/{protocol}/{addr}/tcp/{}", self.addr.port())
            .parse()
            .unwrap()
    }

    pub fn multiaddr_tls(&self) -> Multiaddr {
        let protocol = if self.addr.is_ipv4() { "ip4" } else { "ip6" };
        let addr = self.addr.ip();
        format!("/{protocol}/{addr}/tcp/{}/tls", self.addr.port())
            .parse()
            .unwrap()
    }

    pub fn http_url(&self, path: &str) -> Url {
        format!("http://localhost:{}{path}", self.addr.port())
            .parse()
            .unwrap()
    }

    pub fn https_url(&self, path: &str) -> Url {
        format!("https://localhost:{}{path}", self.addr.port())
            .parse()
            .unwrap()
    }
}
