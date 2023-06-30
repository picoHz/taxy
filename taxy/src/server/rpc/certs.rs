use super::RpcMethod;
use crate::{certs::Cert, server::state::ServerState};
use flate2::{write::GzEncoder, Compression};
use hyper::body::Bytes;
use std::{
    sync::Arc,
    time::{Duration, SystemTime},
};
use tar::Header;
use taxy_api::{cert::CertInfo, error::Error};

pub struct GetCertList;

#[async_trait::async_trait]
impl RpcMethod for GetCertList {
    type Output = Vec<CertInfo>;

    async fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        Ok(state.certs.iter().map(|item| item.info()).collect())
    }
}

pub struct GetCert {
    pub id: String,
}

#[async_trait::async_trait]
impl RpcMethod for GetCert {
    type Output = Arc<Cert>;

    async fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        state
            .certs
            .get(&self.id)
            .cloned()
            .ok_or(Error::IdNotFound { id: self.id })
    }
}

pub struct AddCert {
    pub cert: Arc<Cert>,
}

#[async_trait::async_trait]
impl RpcMethod for AddCert {
    type Output = ();

    async fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        state.certs.add(self.cert.clone())?;
        state.update_certs().await;
        state.storage.save_cert(&self.cert).await;
        Ok(())
    }
}

pub struct DeleteCert {
    pub id: String,
}

#[async_trait::async_trait]
impl RpcMethod for DeleteCert {
    type Output = ();

    async fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        state.certs.delete(&self.id)?;
        state.update_certs().await;
        state.storage.delete_cert(&self.id).await;
        Ok(())
    }
}

pub struct DownloadCert {
    pub id: String,
}

#[async_trait::async_trait]
impl RpcMethod for DownloadCert {
    type Output = Bytes;

    async fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        state
            .certs
            .get(&self.id)
            .map(|cert| cert_to_tar_gz(cert).unwrap())
            .ok_or(Error::IdNotFound { id: self.id })
    }
}

fn cert_to_tar_gz(cert: &Cert) -> anyhow::Result<Bytes> {
    let mut buf = Vec::<u8>::new();

    {
        let enc = GzEncoder::new(&mut buf, Compression::default());
        let mut tar = tar::Builder::new(enc);

        let mut chain = cert.raw_chain.as_slice();
        let mut key = cert.raw_key.as_slice();

        let mtime = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();

        let mut header = Header::new_old();
        header.set_size(chain.len() as _);
        header.set_mtime(mtime);
        header.set_mode(0o644);
        header.set_cksum();
        tar.append_data(&mut header, "chain.pem", &mut chain)?;

        let mut header = Header::new_old();
        header.set_size(key.len() as _);
        header.set_mtime(mtime);
        header.set_mode(0o644);
        header.set_cksum();
        tar.append_data(&mut header, "key.pem", &mut key)?;

        tar.finish()?;
    }

    Ok(buf.into())
}
