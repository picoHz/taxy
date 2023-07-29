use super::storage::Storage;
use crate::certs::{
    acme::{AcmeAccount, AcmeEntry},
    Cert,
};
use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use indexmap::map::IndexMap;
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::Arc,
};
use taxy_api::{
    app::AppConfig,
    auth::{Account, LoginMethod, LoginRequest, LoginResponse},
    cert::CertKind,
    id::ShortId,
};
use taxy_api::{
    error::Error,
    port::{Port, PortEntry},
    site::{Proxy, ProxyEntry},
};
use tokio::fs;
use tokio::io::AsyncReadExt;
use toml_edit::Document;
use totp_rs::{Secret, TOTP};
use tracing::{error, info, warn};

pub struct FileStorage {
    dir: PathBuf,
}

impl FileStorage {
    pub fn new(dir: &Path) -> Self {
        Self {
            dir: dir.to_owned(),
        }
    }

    async fn save_app_config_impl(&self, path: &Path, config: &AppConfig) -> anyhow::Result<()> {
        fs::create_dir_all(path.parent().unwrap()).await?;
        info!(?path, "save config");
        fs::write(path, toml::to_string(config)?).await?;
        Ok(())
    }

    async fn load_app_config_impl(&self, path: &Path) -> anyhow::Result<AppConfig> {
        info!(?path, "load config");
        let content = fs::read_to_string(path).await?;
        Ok(toml::from_str(&content)?)
    }

    async fn save_ports_impl(&self, path: &Path, ports: &[PortEntry]) -> anyhow::Result<()> {
        fs::create_dir_all(path.parent().unwrap()).await?;
        info!(?path, "save config");
        let mut doc = match self.load_document(path).await {
            Ok(doc) => doc,
            Err(err) => {
                warn!(?path, ?err, "failed to load config");
                Document::new()
            }
        };

        let mut unused = doc
            .as_table()
            .iter()
            .map(|(key, _)| key.to_string())
            .collect::<HashSet<_>>();
        for port in ports {
            let (id, entry): (ShortId, Port) = port.clone().into();
            let id = id.to_string();
            doc[&id].clone_from(toml_edit::ser::to_document(&entry)?.as_item());
            unused.remove(&id);
        }
        for key in unused {
            doc.remove(&key);
        }

        fs::write(path, doc.to_string()).await?;
        Ok(())
    }

    async fn load_document(&self, path: &Path) -> anyhow::Result<Document> {
        info!(?path, "load config");
        let content = fs::read_to_string(path).await?;
        Ok(content.parse::<Document>()?)
    }

    async fn load_ports_impl(&self, path: &Path) -> anyhow::Result<Vec<PortEntry>> {
        info!(?path, "load config");
        let content = fs::read_to_string(path).await?;
        let table: IndexMap<ShortId, Port> = toml::from_str(&content)?;
        Ok(table.into_iter().map(|entry| entry.into()).collect())
    }

    async fn load_proxies_impl(&self, path: &Path) -> anyhow::Result<Vec<ProxyEntry>> {
        info!(?path, "load proxies");
        let content = fs::read_to_string(path).await?;
        let table: IndexMap<ShortId, Proxy> = toml::from_str(&content)?;
        Ok(table.into_iter().map(|entry| entry.into()).collect())
    }

    async fn save_proxies_impl(&self, path: &Path, proxies: &[ProxyEntry]) -> anyhow::Result<()> {
        fs::create_dir_all(path.parent().unwrap()).await?;
        info!(?path, "save config");
        let mut doc = match self.load_document(path).await {
            Ok(doc) => doc,
            Err(err) => {
                warn!(?path, ?err, "failed to load config");
                Document::new()
            }
        };

        let mut unused = doc
            .as_table()
            .iter()
            .map(|(key, _)| key.to_string())
            .collect::<HashSet<_>>();
        for site in proxies {
            let (id, entry): (ShortId, Proxy) = site.clone().into();
            let id = id.to_string();
            doc[&id].clone_from(toml_edit::ser::to_document(&entry)?.as_item());
            unused.remove(&id);
        }
        for key in unused {
            doc.remove(&key);
        }

        fs::write(path, doc.to_string()).await?;
        Ok(())
    }

    async fn save_cert_impl(&self, path: &Path, cert: &Cert) -> anyhow::Result<()> {
        fs::create_dir_all(path).await?;
        info!(?path, "save cert");
        fs::write(path.join("cert.pem"), &cert.pem_chain).await?;
        if let Some(key) = &cert.pem_key {
            fs::write(path.join("key.pem"), key).await?;
        }
        Ok(())
    }

    async fn save_acme_impl(&self, path: &Path, acme: &AcmeEntry) -> anyhow::Result<()> {
        fs::create_dir_all(path.parent().unwrap()).await?;
        info!(?path, "save config");
        let mut doc = match self.load_document(path).await {
            Ok(doc) => doc,
            Err(err) => {
                warn!(?path, ?err, "failed to load config");
                Document::new()
            }
        };

        let (id, entry): (ShortId, AcmeAccount) = acme.clone().into();
        let id = id.to_string();
        doc[&id].clone_from(toml_edit::ser::to_document(&entry)?.as_item());

        fs::write(path, doc.to_string()).await?;
        Ok(())
    }

    async fn delete_acme_impl(&self, path: &Path, id: ShortId) -> anyhow::Result<()> {
        info!(?path, "delete acme");
        let mut doc = match self.load_document(path).await {
            Ok(doc) => doc,
            Err(err) => {
                warn!(?path, ?err, "failed to load config");
                Document::new()
            }
        };

        doc.remove(&id.to_string());
        fs::write(path, doc.to_string()).await?;
        Ok(())
    }

    pub async fn load_certs_impl(
        &self,
        path: &Path,
        kind: CertKind,
    ) -> anyhow::Result<Vec<Arc<Cert>>> {
        let walker = globwalk::GlobWalkerBuilder::from_patterns(
            path.join(kind.to_string()),
            &["*/cert.pem"],
        )
        .build()?
        .filter_map(Result::ok);

        let mut certs = Vec::new();
        for pem in walker {
            let chain = pem.path();
            let key = pem.path().parent().unwrap().join("key.pem");
            let mut chain_data = Vec::new();
            let mut key_data = Vec::new();

            match fs::File::open(&chain).await {
                Ok(mut file) => {
                    if let Err(err) = file.read_to_end(&mut chain_data).await {
                        error!(path = ?chain, "failed to load: {err}");
                    }
                }
                Err(err) => {
                    error!(path = ?chain, "failed to load: {err}");
                }
            }

            match fs::File::open(&key).await {
                Ok(mut file) => {
                    if let Err(err) = file.read_to_end(&mut key_data).await {
                        error!(path = ?key, "failed to load: {err}");
                    }
                }
                Err(err) => {
                    error!(path = ?key, "failed to load: {err}");
                }
            }

            let key_data = if key_data.is_empty() {
                None
            } else {
                Some(key_data)
            };

            match Cert::new(kind, chain_data, key_data) {
                Ok(cert) => certs.push(Arc::new(cert)),
                Err(err) => error!(?path, "failed to load: {err}"),
            }
        }
        Ok(certs)
    }

    pub async fn load_acmes_impl(&self, path: &Path) -> anyhow::Result<Vec<AcmeEntry>> {
        info!(?path, "load acmes");
        let content = fs::read_to_string(path).await?;
        let table: IndexMap<ShortId, AcmeAccount> = toml::from_str(&content)?;
        Ok(table.into_iter().map(|entry| entry.into()).collect())
    }

    async fn add_account_impl(
        &self,
        name: &str,
        password: &str,
        totp: bool,
    ) -> anyhow::Result<Account> {
        fs::create_dir_all(&self.dir).await?;
        let path = self.dir.join("accounts.toml");
        info!(?path, "save account");

        let mut doc = match fs::read_to_string(&path).await {
            Ok(content) => content.parse::<Document>().unwrap_or_default(),
            Err(_) => Document::default(),
        };

        let salt = SaltString::generate(rand::thread_rng());
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|_| anyhow::anyhow!("failed to hash password"))?
            .to_string();

        let account = Account {
            password: password_hash,
            totp: if totp {
                Some(TOTP::default().get_secret_base32())
            } else {
                None
            },
        };
        doc[name].clone_from(toml_edit::ser::to_document(&account)?.as_item());

        fs::write(&path, doc.to_string()).await?;
        Ok(account)
    }

    async fn load_accounts(&self) -> anyhow::Result<HashMap<String, Account>> {
        let path = self.dir.join("accounts.toml");
        info!(?path, "load accounts");
        let content = fs::read_to_string(&path).await?;
        Ok(toml::from_str(&content)?)
    }

    async fn verify_password(&self, name: &str, password: &str) -> Result<LoginResponse, Error> {
        let accounts = match self.load_accounts().await {
            Ok(accounts) => accounts,
            Err(err) => {
                error!(?err, "failed to load accounts: {err}");
                return Err(Error::InvalidLoginCredentials);
            }
        };

        let account = match accounts.get(name) {
            Some(account) => account,
            None => {
                error!(?name, "account not found: {name}");
                return Err(Error::InvalidLoginCredentials);
            }
        };

        let parsed_hash = match PasswordHash::new(&account.password) {
            Ok(parsed_hash) => parsed_hash,
            Err(err) => {
                error!(?err, "failed to parse password hash: {err}");
                return Err(Error::InvalidLoginCredentials);
            }
        };

        let argon2 = Argon2::default();
        if let Err(err) = argon2.verify_password(password.as_bytes(), &parsed_hash) {
            error!(?err, "failed to verify password: {err}");
            return Err(Error::InvalidLoginCredentials);
        }

        if account.totp.is_some() {
            return Ok(LoginResponse::TotpRequired);
        }

        Ok(LoginResponse::Success)
    }

    async fn verify_totp(&self, name: &str, token: &str) -> Result<LoginResponse, Error> {
        let accounts = match self.load_accounts().await {
            Ok(accounts) => accounts,
            Err(err) => {
                error!(?err, "failed to load accounts: {err}");
                return Err(Error::InvalidLoginCredentials);
            }
        };

        let account = match accounts.get(name) {
            Some(account) => account,
            None => {
                error!(?name, "account not found: {name}");
                return Err(Error::InvalidLoginCredentials);
            }
        };

        let secret = match &account.totp {
            Some(totp) => Secret::Encoded(totp.clone())
                .to_bytes()
                .map_err(|_| Error::InvalidLoginCredentials)?,
            None => {
                error!(?name, "totp not found: {name}");
                return Err(Error::InvalidLoginCredentials);
            }
        };

        let totp = TOTP {
            secret,
            ..Default::default()
        };

        if totp.check_current(token).unwrap_or_default() {
            return Ok(LoginResponse::Success);
        }
        Err(Error::InvalidLoginCredentials)
    }
}

#[async_trait::async_trait]
impl Storage for FileStorage {
    async fn save_app_config(&self, config: &AppConfig) {
        let dir = &self.dir;
        let path = dir.join("config.toml");
        if let Err(err) = self.save_app_config_impl(&path, config).await {
            error!(?path, "failed to save: {err}");
        }
    }

    async fn load_app_config(&self) -> AppConfig {
        let dir = &self.dir;
        let path = dir.join("config.toml");
        match self.load_app_config_impl(&path).await {
            Ok(config) => config,
            Err(err) => {
                warn!(?path, "failed to load: {err}");
                Default::default()
            }
        }
    }

    async fn save_ports(&self, entries: &[PortEntry]) {
        let dir = &self.dir;
        let path = dir.join("ports.toml");
        if let Err(err) = self.save_ports_impl(&path, entries).await {
            error!(?path, "failed to save: {err}");
        }
    }

    async fn load_ports(&self) -> Vec<PortEntry> {
        let dir = &self.dir;
        let path = dir.join("ports.toml");
        match self.load_ports_impl(&path).await {
            Ok(ports) => ports,
            Err(err) => {
                warn!(?path, "failed to load: {err}");
                Default::default()
            }
        }
    }

    async fn save_proxies(&self, proxies: &[ProxyEntry]) {
        let dir = &self.dir;
        let path = dir.join("proxies.toml");
        if let Err(err) = self.save_proxies_impl(&path, proxies).await {
            error!(?path, "failed to save: {err}");
        }
    }

    async fn load_proxies(&self) -> Vec<ProxyEntry> {
        let dir = &self.dir;
        let path = dir.join("proxies.toml");
        match self.load_proxies_impl(&path).await {
            Ok(proxies) => proxies,
            Err(err) => {
                warn!(?path, "failed to load: {err}");
                Default::default()
            }
        }
    }

    async fn save_cert(&self, cert: &Cert) {
        let dir = &self.dir;
        let path = dir
            .join("certs")
            .join(cert.kind.to_string())
            .join(cert.id().to_string());
        if let Err(err) = self.save_cert_impl(&path, cert).await {
            error!(?path, "failed to save: {err}");
        }
    }

    async fn save_acme(&self, acme: &AcmeEntry) {
        let path = self.dir.join("acme.toml");
        if let Err(err) = self.save_acme_impl(&path, acme).await {
            error!(?path, "failed to save: {err}");
        }
    }

    async fn delete_acme(&self, id: ShortId) {
        let path = self.dir.join("acme.toml");
        if let Err(err) = self.delete_acme_impl(&path, id).await {
            error!(?path, "failed to delete: {err}");
        }
    }

    async fn delete_cert(&self, id: ShortId) {
        let dir = &self.dir;

        if let Ok(walker) =
            globwalk::GlobWalkerBuilder::from_patterns(dir.join("certs"), &[&format!("*/{id}")])
                .build()
        {
            for entry in walker.filter_map(Result::ok) {
                if let Err(err) = fs::remove_dir_all(entry.path()).await {
                    error!(path = ?entry.path(), "failed to delete: {err}");
                }
            }
        }
    }

    async fn load_acmes(&self) -> Vec<AcmeEntry> {
        let dir = &self.dir;
        let path = dir.join("acme.toml");
        match self.load_acmes_impl(&path).await {
            Ok(acmes) => acmes,
            Err(err) => {
                warn!(?path, "failed to load: {err}");
                Default::default()
            }
        }
    }

    async fn load_certs(&self) -> Vec<Arc<Cert>> {
        let dir = &self.dir;
        let path = dir.join("certs");
        let mut certs = Vec::new();
        match self.load_certs_impl(&path, CertKind::Server).await {
            Ok(mut entries) => certs.append(&mut entries),
            Err(err) => {
                warn!(?path, "failed to load: {err}");
            }
        }
        match self.load_certs_impl(&path, CertKind::Root).await {
            Ok(mut entries) => certs.append(&mut entries),
            Err(err) => {
                warn!(?path, "failed to load: {err}");
            }
        }
        certs
    }

    async fn add_account(&self, name: &str, password: &str, totp: bool) -> Result<Account, Error> {
        self.add_account_impl(name, password, totp)
            .await
            .map_err(|_| Error::FailedToCreateAccount)
    }

    async fn verify_account(&self, request: LoginRequest) -> Result<LoginResponse, Error> {
        match request.method {
            LoginMethod::Password { password } => {
                self.verify_password(&request.username, &password).await
            }
            LoginMethod::Totp { token } => self.verify_totp(&request.username, &token).await,
        }
    }
}
