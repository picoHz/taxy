use super::{port::PortEntry, AppConfig};
use crate::{certs::Cert, config::port::NamelessPortEntry};
use indexmap::map::IndexMap;
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};
use tokio::fs;
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
            let (name, entry): (String, NamelessPortEntry) = port.clone().into();
            doc[&name] = toml_edit::ser::to_document(&entry)?.as_item().clone();
            unused.remove(&name);
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
        let table: IndexMap<String, NamelessPortEntry> = toml::from_str(&content)?;
        Ok(table.into_iter().map(|entry| entry.into()).collect())
    }

    pub async fn save_cert(&self, cert: &Cert) {
        let dir = &self.dir;
        let path = dir.join("certs").join(&cert.info.id);
        if let Err(err) = self.save_cert_impl(&path, cert).await {
            error!(?path, "failed to save: {err}");
        }
    }

    async fn save_cert_impl(&self, path: &Path, cert: &Cert) -> anyhow::Result<()> {
        fs::create_dir_all(path).await?;
        info!(?path, "save cert");
        fs::write(path.join("cert.pem"), &cert.raw_chain).await?;
        fs::write(path.join("key.pem"), &cert.raw_key).await?;
        Ok(())
    }
}
