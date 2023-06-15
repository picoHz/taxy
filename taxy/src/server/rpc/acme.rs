use super::RpcMethod;
use crate::server::state::ServerState;
use taxy_api::{
    acme::{AcmeInfo, AcmeRequest},
    error::Error,
};

pub struct GetAcmeList;

#[async_trait::async_trait]
impl RpcMethod for GetAcmeList {
    type Output = Vec<AcmeInfo>;

    async fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        Ok(state.get_acme_list())
    }
}

pub struct GetAcme {
    pub id: String,
}

#[async_trait::async_trait]
impl RpcMethod for GetAcme {
    type Output = AcmeInfo;

    async fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        state.get_acme(&self.id)
    }
}

pub struct AddAcme {
    pub request: AcmeRequest,
}

#[async_trait::async_trait]
impl RpcMethod for AddAcme {
    type Output = ();

    async fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        state.add_acme(self.request).await
    }
}

pub struct DeleteAcme {
    pub id: String,
}

#[async_trait::async_trait]
impl RpcMethod for DeleteAcme {
    type Output = ();

    async fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        state.delete_keyring_item(&self.id).await
    }
}
