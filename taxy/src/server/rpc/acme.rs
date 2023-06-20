use super::RpcMethod;
use crate::{keyring::acme::AcmeEntry, server::state::ServerState};
use taxy_api::{
    acme::{AcmeInfo, AcmeRequest},
    error::Error,
};

pub struct GetAcmeList;

#[async_trait::async_trait]
impl RpcMethod for GetAcmeList {
    type Output = Vec<AcmeInfo>;

    async fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        Ok(state.acmes.entries().map(|acme| acme.info()).collect())
    }
}

pub struct GetAcme {
    pub id: String,
}

#[async_trait::async_trait]
impl RpcMethod for GetAcme {
    type Output = AcmeInfo;

    async fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        state
            .acmes
            .get(&self.id)
            .map(|acme| acme.info())
            .ok_or(Error::IdNotFound { id: self.id })
    }
}

pub struct AddAcme {
    pub request: AcmeRequest,
}

#[async_trait::async_trait]
impl RpcMethod for AddAcme {
    type Output = ();

    async fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        let entry = AcmeEntry::new(state.generate_id(), self.request).await?;
        state.acmes.add(entry.clone())?;
        state.storage.save_acme(&entry).await;
        state.update_acmes().await;
        Ok(())
    }
}

pub struct DeleteAcme {
    pub id: String,
}

#[async_trait::async_trait]
impl RpcMethod for DeleteAcme {
    type Output = ();

    async fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        state.acmes.delete(&self.id)?;
        state.update_acmes().await;
        state.storage.delete_acme(&self.id).await;
        Ok(())
    }
}
