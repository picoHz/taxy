use super::RpcMethod;
use crate::{keyring::acme::AcmeEntry, server::state::ServerState};
use taxy_api::{acme::AcmeInfo, error::Error};

pub struct GetAcmeList;

#[async_trait::async_trait]
impl RpcMethod for GetAcmeList {
    type Output = Vec<AcmeInfo>;

    async fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        Ok(state.get_acme_list())
    }
}

pub struct AddAcme {
    pub item: AcmeEntry,
}

#[async_trait::async_trait]
impl RpcMethod for AddAcme {
    type Output = ();

    async fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        state.add_acme(self.item).await
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
