use super::RpcMethod;
use crate::{keyring::certs::Cert, server::state::ServerState};
use taxy_api::{cert::CertInfo, error::Error};

pub struct GetServerCertList;

#[async_trait::async_trait]
impl RpcMethod for GetServerCertList {
    type Output = Vec<CertInfo>;

    async fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        Ok(state.get_server_cert_list())
    }
}

pub struct GetServerCert {
    pub id: String,
}

#[async_trait::async_trait]
impl RpcMethod for GetServerCert {
    type Output = CertInfo;

    async fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        state.get_server_cert(&self.id)
    }
}

pub struct AddServerCert {
    pub cert: Cert,
}

#[async_trait::async_trait]
impl RpcMethod for AddServerCert {
    type Output = ();

    async fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        state.add_server_cert(self.cert).await
    }
}

pub struct DeleteServerCert {
    pub id: String,
}

#[async_trait::async_trait]
impl RpcMethod for DeleteServerCert {
    type Output = ();

    async fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        state.delete_keyring_item(&self.id).await
    }
}
