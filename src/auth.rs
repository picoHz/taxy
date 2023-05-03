use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path};
use tokio::fs;
use toml_edit::Document;
use tracing::info;

use crate::error;

pub async fn add_account(config_dir: &Path, name: &str, password: &str) -> anyhow::Result<()> {
    fs::create_dir_all(&config_dir).await?;

    let path = config_dir.join("accounts.toml");
    info!(?path, "save account");

    let mut doc = match fs::read_to_string(&path).await {
        Ok(content) => content.parse::<Document>().unwrap_or_default(),
        Err(_) => Document::default(),
    };

    let salt = SaltString::generate(&mut OsRng);
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

async fn load_accounts(config_dir: &Path) -> anyhow::Result<HashMap<String, Account>> {
    let path = config_dir.join("accounts.toml");
    info!(?path, "load accounts");
    let content = fs::read_to_string(&path).await?;
    Ok(toml::from_str(&content)?)
}

pub async fn verify_account(config_dir: &Path, name: &str, password: &str) -> bool {
    let accounts = match load_accounts(config_dir).await {
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

#[derive(Clone, Serialize, Deserialize)]
pub struct Account {
    pub password: String,
}
