use super::storage::Storage;
use crate::certs::{
    acme::{AcmeAccount, AcmeEntry},
    Cert,
};
use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use indexmap::map::IndexMap;
use serde_derive::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::Arc,
};
use taxy_api::{app::AppConfig, cert::CertKind};
use taxy_api::{
    error::Error,
    port::{Port, PortEntry},
    site::{Site, SiteEntry},
};
use tokio::fs;
use tokio::io::AsyncReadExt;
use toml_edit::Document;
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
            let (id, entry): (String, Port) = port.clone().into();
            doc[&id] = toml_edit::ser::to_document(&entry)?.as_item().clone();
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
        let table: IndexMap<String, Port> = toml::from_str(&content)?;
        Ok(table.into_iter().map(|entry| entry.into()).collect())
    }

    async fn load_sites_impl(&self, path: &Path) -> anyhow::Result<Vec<SiteEntry>> {
        info!(?path, "load sites");
        let content = fs::read_to_string(path).await?;
        let table: IndexMap<String, Site> = toml::from_str(&content)?;
        Ok(table.into_iter().map(|entry| entry.into()).collect())
    }

    async fn save_sites_impl(&self, path: &Path, sites: &[SiteEntry]) -> anyhow::Result<()> {
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
        for site in sites {
            let (id, entry): (String, Site) = site.clone().into();
            doc[&id] = toml_edit::ser::to_document(&entry)?.as_item().clone();
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
        fs::write(path.join("key.pem"), &cert.pem_key).await?;
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

        let (id, entry): (String, AcmeAccount) = acme.clone().into();
        doc[&id] = toml_edit::ser::to_document(&entry)?.as_item().clone();

        fs::write(path, doc.to_string()).await?;
        Ok(())
    }

    async fn delete_acme_impl(&self, path: &Path, id: &str) -> anyhow::Result<()> {
        info!(?path, "delete acme");
        let mut doc = match self.load_document(path).await {
            Ok(doc) => doc,
            Err(err) => {
                warn!(?path, ?err, "failed to load config");
                Document::new()
            }
        };

        doc.remove(id);
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
        let table: IndexMap<String, AcmeAccount> = toml::from_str(&content)?;
        Ok(table.into_iter().map(|entry| entry.into()).collect())
    }

    async fn add_account_impl(&self, name: &str, password: &str) -> anyhow::Result<()> {
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
        };
        doc[name] = toml_edit::ser::to_document(&account)?.as_item().clone();

        fs::write(&path, doc.to_string()).await?;
        Ok(())
    }

    async fn load_accounts(&self) -> anyhow::Result<HashMap<String, Account>> {
        let path = self.dir.join("accounts.toml");
        info!(?path, "load accounts");
        let content = fs::read_to_string(&path).await?;
        Ok(toml::from_str(&content)?)
    }

    async fn verify_account(&self, name: &str, password: &str) -> bool {
        let accounts = match self.load_accounts().await {
            Ok(accounts) => accounts,
            Err(err) => {
                error!(?err, "failed to load accounts: {err}");
                return false;
            }
        };

        let account = match accounts.get(name) {
            Some(account) => account,
            None => {
                error!(?name, "account not found: {name}");
                return false;
            }
        };

        let parsed_hash = match PasswordHash::new(&account.password) {
            Ok(parsed_hash) => parsed_hash,
            Err(err) => {
                error!(?err, "failed to parse password hash: {err}");
                return false;
            }
        };

        let argon2 = Argon2::default();
        if let Err(err) = argon2.verify_password(password.as_bytes(), &parsed_hash) {
            error!(?err, "failed to verify password: {err}");
            return false;
        }

        true
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

    async fn save_sites(&self, sites: &[SiteEntry]) {
        let dir = &self.dir;
        let path = dir.join("sites.toml");
        if let Err(err) = self.save_sites_impl(&path, sites).await {
            error!(?path, "failed to save: {err}");
        }
    }

    async fn load_sites(&self) -> Vec<SiteEntry> {
        let dir = &self.dir;
        let path = dir.join("sites.toml");
        match self.load_sites_impl(&path).await {
            Ok(sites) => sites,
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
            .join(cert.id());
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

    async fn delete_acme(&self, id: &str) {
        let path = self.dir.join("acme.toml");
        if let Err(err) = self.delete_acme_impl(&path, id).await {
            error!(?path, "failed to delete: {err}");
        }
    }

    async fn delete_cert(&self, id: &str) {
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

    async fn add_account(&self, name: &str, password: &str) -> Result<(), Error> {
        self.add_account_impl(name, password)
            .await
            .map_err(|_| Error::FailedToCreateAccount)
    }

    async fn verify_account(&self, name: &str, password: &str) -> Result<(), Error> {
        if self.verify_account(name, password).await {
            Ok(())
        } else {
            Err(Error::InvalidLoginCredentials)
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
struct Account {
    pub password: String,
}
