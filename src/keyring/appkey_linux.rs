#![cfg(target_os = "linux")]

pub fn init_appkey(use_keystore: bool) -> anyhow::Result<()> {
    if use_keystore {
        Err(anyhow::anyhow!("not implemented"))
    } else {
        Ok(())
    }
}

pub fn load_appkey() -> anyhow::Result<Option<Zeroizing<Vec<u8>>>> {
    Ok(None)
}
