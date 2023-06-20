use super::RpcMethod;
use crate::{keyring::certs::Cert, server::state::ServerState};
use std::sync::Arc;
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
