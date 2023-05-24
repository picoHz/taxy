use super::{RpcMethod};
use crate::{
    error::Error,
    keyring::acme::{AcmeEntry, AcmeInfo},
    server::state::ServerState,
};

pub struct GetAcmeList;

#[async_trait::async_trait]
impl RpcMethod for GetAcmeList {
    type Output = Vec<AcmeInfo>;

    async fn call(&self, state: &mut ServerState) -> Result<Self::Output, Error> {
        Ok(state.get_acme_list())
    }
}

pub struct AddAcme {
    pub item: AcmeEntry,
}

#[async_trait::async_trait]
impl RpcMethod for AddAcme {
    type Output = ();

    async fn call(&self, state: &mut ServerState) -> Result<Self::Output, Error> {
        state.add_acme(self.item.clone())
    }
}

pub struct DeleteAcme {
    pub id: String,
}

#[async_trait::async_trait]
impl RpcMethod for DeleteAcme {
    type Output = ();

    async fn call(&self, state: &mut ServerState) -> Result<Self::Output, Error> {
        state.delete_acme(&self.id)
    }
}
