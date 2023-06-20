use super::RpcMethod;
use crate::{keyring::certs::Cert, server::state::ServerState};
use flate2::{write::GzEncoder, Compression};
use hyper::body::Bytes;
use std::sync::Arc;
use tar::Header;
use taxy_api::{cert::CertInfo, error::Error};

pub struct GetServerCertList;

#[async_trait::async_trait]
impl RpcMethod for GetServerCertList {
    type Output = Vec<CertInfo>;

    async fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        Ok(state.certs.iter().map(|item| item.info()).collect())
    }
}

pub struct GetServerCert {
    pub id: String,
}

#[async_trait::async_trait]
impl RpcMethod for GetServerCert {
    type Output = CertInfo;

    async fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        state
            .certs
            .get(&self.id)
            .map(|item| item.info())
            .ok_or(Error::IdNotFound { id: self.id })
    }
}

pub struct AddServerCert {
    pub cert: Cert,
}

#[async_trait::async_trait]
impl RpcMethod for AddServerCert {
    type Output = ();

    async fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        let cert = Arc::new(self.cert);
        state.certs.add(cert.clone())?;
        state.update_certs().await;
        state.storage.save_cert(&cert).await;
        Ok(())
    }
}

pub struct DeleteServerCert {
    pub id: String,
}

#[async_trait::async_trait]
impl RpcMethod for DeleteServerCert {
    type Output = ();

    async fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        state.certs.delete(&self.id)?;
        state.storage.delete_cert(&self.id).await;
        Ok(())
    }
}

pub struct DownloadServerCert {
    pub id: String,
}

#[async_trait::async_trait]
impl RpcMethod for DownloadServerCert {
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

        let mut header = Header::new_old();
        header.set_size(chain.len() as _);
        header.set_mode(0o644);
        header.set_cksum();
        tar.append_data(&mut header, "chain.pem", &mut chain)?;

        let mut header = Header::new_old();
        header.set_size(key.len() as _);
        header.set_mode(0o644);
        header.set_cksum();
        tar.append_data(&mut header, "key.pem", &mut key)?;

        tar.finish()?;
    }

    Ok(buf.into())
}
