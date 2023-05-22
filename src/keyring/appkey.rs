#![cfg(not(target_os = "linux"))]

use base64::{engine::general_purpose, Engine as _};
use log::info;
use once_cell::sync::OnceCell;
use rand::RngCore;
use zeroize::Zeroizing;

const APPKEY_KEYRING_SERVICE: &str = "taxy.encryption";
const APPKEY_KEYRING_USER: &str = "taxy";
const APPKEY_LENGTH: usize = 32;

static ENTRY: OnceCell<Option<keyring::Entry>> = OnceCell::new();

pub fn init_appkey(use_keystore: bool) -> anyhow::Result<()> {
    ENTRY.get_or_try_init(|| {
        if use_keystore {
            keyring::Entry::new(APPKEY_KEYRING_SERVICE, APPKEY_KEYRING_USER).map(Some)
        } else {
            Ok(None)
        }
    })?;
    Ok(())
}

pub fn load_appkey() -> anyhow::Result<Option<Zeroizing<Vec<u8>>>> {
    let entry = if let Some(entry) = ENTRY.get().and_then(|entry| entry.as_ref()) {
        entry
    } else {
        return Ok(None);
    };
    match entry.get_password() {
        Ok(password) => Ok(Some(Zeroizing::new(
            general_purpose::STANDARD_NO_PAD.decode::<&str>(&password)?,
        ))),
        Err(keyring::Error::NoEntry) => {
            info!("generating appkey...");
            let mut key = Zeroizing::new(vec![0u8; APPKEY_LENGTH]);
            rand::thread_rng().fill_bytes(key.as_mut());
            let password = Zeroizing::new(general_purpose::STANDARD_NO_PAD.encode(&key));
            entry.set_password(&password)?;
            Ok(Some(key))
        }
        Err(err) => Err(err.into()),
    }
}
