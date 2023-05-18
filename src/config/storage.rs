use super::{port::PortEntry, AppConfig};
use crate::{
    config::port::Port,
    keyring::{
        acme::{AcmeAccount, AcmeEntry},
        certs::Cert,
        {Keyring, KeyringItem},
    },
};
use argon2::password_hash::rand_core::OsRng;
use indexmap::map::IndexMap;
use pkcs8::{EncryptedPrivateKeyInfo, PrivateKeyInfo, SecretDocument};
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::fs;
use tokio::io::AsyncReadExt;
use toml_edit::Document;
use tracing::{error, info, warn};

pub struct ConfigStorage {
    dir: PathBuf,
}

impl ConfigStorage {
    pub fn new(dir: &Path) -> Self {
        Self {
            dir: dir.to_owned(),
        }
    }

    pub async fn save_app_config(&self, config: &AppConfig) {
        let dir = &self.dir;
        let path = dir.join("config.toml");
        if let Err(err) = self.save_app_config_impl(&path, config).await {
            error!(?path, "failed to save: {err}");
        }
    }

    async fn save_app_config_impl(&self, path: &Path, config: &AppConfig) -> anyhow::Result<()> {
        fs::create_dir_all(path.parent().unwrap()).await?;
        info!(?path, "save config");
        fs::write(path, toml::to_string(config)?).await?;
        Ok(())
    }

    pub async fn load_app_config(&self) -> AppConfig {
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

    async fn load_app_config_impl(&self, path: &Path) -> anyhow::Result<AppConfig> {
        info!(?path, "load config");
        let content = fs::read_to_string(path).await?;
        Ok(toml::from_str(&content)?)
    }

    pub async fn save_entries(&self, entries: &[PortEntry]) {
        let dir = &self.dir;
        let path = dir.join("ports.toml");
        if let Err(err) = self.save_entries_impl(&path, entries).await {
            error!(?path, "failed to save: {err}");
        }
    }

    async fn save_entries_impl(&self, path: &Path, ports: &[PortEntry]) -> anyhow::Result<()> {
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

    pub async fn load_entries(&self) -> Vec<PortEntry> {
        let dir = &self.dir;
        let path = dir.join("ports.toml");
        match self.load_entries_impl(&path).await {
            Ok(ports) => ports,
            Err(err) => {
                warn!(?path, "failed to load: {err}");
                Default::default()
            }
        }
    }

    async fn load_entries_impl(&self, path: &Path) -> anyhow::Result<Vec<PortEntry>> {
        info!(?path, "load config");
        let content = fs::read_to_string(path).await?;
        let table: IndexMap<String, Port> = toml::from_str(&content)?;
        Ok(table.into_iter().map(|entry| entry.into()).collect())
    }

    pub async fn save_cert(&self, cert: &Cert) {
        let dir = &self.dir;
        let path = dir.join("certs").join(cert.id());
        if let Err(err) = self.save_cert_impl(&path, cert).await {
            error!(?path, "failed to save: {err}");
        }
    }

    async fn save_cert_impl(&self, path: &Path, cert: &Cert) -> anyhow::Result<()> {
        fs::create_dir_all(path).await?;
        info!(?path, "save cert");

        let (_, doc) = SecretDocument::from_pem(&std::str::from_utf8(&cert.raw_key)?)
            .map_err(|_| anyhow::anyhow!("failed to parse pem"))?;
        let key_info: PrivateKeyInfo = doc
            .decode_msg()
            .map_err(|_| anyhow::anyhow!("failed to parse private key info"))?;
        let secret_doc = key_info
            .encrypt(OsRng, "password")
            .map_err(|_| anyhow::anyhow!("failed to encrypt private key info"))?;
        let encrypted_key_pem = secret_doc
            .to_pem("ENCRYPTED PRIVATE KEY", pkcs8::LineEnding::CRLF)
            .map_err(|_| anyhow::anyhow!("failed to encrypt private key info"))?;

        fs::write(path.join("cert.pem"), &cert.raw_chain).await?;
        fs::write(path.join("key.pem"), encrypted_key_pem.as_bytes()).await?;
        Ok(())
    }

    pub async fn save_acme(&self, acme: &AcmeEntry) {
        let path = self.dir.join("acme.toml");
        if let Err(err) = self.save_acme_impl(&path, acme).await {
            error!(?path, "failed to save: {err}");
        }
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

    pub async fn delete_acme(&self, id: &str) {
        let path = self.dir.join("acme.toml");
        if let Err(err) = self.delete_acme_impl(&path, id).await {
            error!(?path, "failed to save: {err}");
        }
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

    pub async fn delete_cert(&self, id: &str) {
        let dir = &self.dir;
        let path = dir.join("certs").join(id);
        if let Err(err) = fs::remove_dir_all(&path).await {
            error!(?path, "failed to delete: {err}");
        }
    }

    pub async fn load_keychain(&self) -> Keyring {
        let mut items = Vec::new();

        let path = self.dir.join("certs");
        match self.load_certs_impl(&path).await {
            Ok(mut certs) => items.append(&mut certs),
            Err(err) => {
                warn!(?path, "failed to load certs: {err}");
            }
        }

        let path = self.dir.join("acme.toml");
        match self.load_acmes_impl(&path).await {
            Ok(mut certs) => items.append(&mut certs),
            Err(err) => {
                warn!(?path, "failed to load acme config: {err}");
            }
        }

        Keyring::new(items)
    }

    pub async fn load_certs_impl(&self, path: &Path) -> anyhow::Result<Vec<KeyringItem>> {
        let walker = globwalk::GlobWalkerBuilder::from_patterns(path, &["*/cert.pem"])
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

            let (_, doc) = SecretDocument::from_pem(&std::str::from_utf8(&key_data)?)
                .map_err(|_| anyhow::anyhow!("failed to parse pem"))?;
            let key_info: EncryptedPrivateKeyInfo = doc
                .decode_msg()
                .map_err(|_| anyhow::anyhow!("failed to parse private key info"))?;
            let secret_doc = key_info
                .decrypt("password")
                .map_err(|_| anyhow::anyhow!("failed to encrypt private key info"))?;
            let decrypted_key_pem = secret_doc
                .to_pem("PRIVATE KEY", pkcs8::LineEnding::CRLF)
                .map_err(|_| anyhow::anyhow!("failed to encrypt private key info"))?;

            match Cert::new(chain_data, decrypted_key_pem.as_bytes().to_vec()) {
                Ok(cert) => certs.push(KeyringItem::ServerCert(Arc::new(cert))),
                Err(err) => error!(?path, "failed to load: {err}"),
            }
        }
        Ok(certs)
    }

    pub async fn load_acmes_impl(&self, path: &Path) -> anyhow::Result<Vec<KeyringItem>> {
        info!(?path, "load acmes");
        let content = fs::read_to_string(path).await?;
        let table: IndexMap<String, AcmeAccount> = toml::from_str(&content)?;
        Ok(table
            .into_iter()
            .map(|entry| KeyringItem::Acme(Arc::new(entry.into())))
            .collect())
    }
}
