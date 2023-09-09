use super::RpcMethod;
use crate::{certs::acme::AcmeEntry, server::state::ServerState};
use taxy_api::{
    acme::{AcmeConfig, AcmeInfo, AcmeRequest},
    error::Error,
    id::ShortId,
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
    pub id: ShortId,
}

#[async_trait::async_trait]
impl RpcMethod for GetAcme {
    type Output = AcmeInfo;

    async fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        state
            .acmes
            .get(self.id)
            .map(|acme| acme.info())
            .ok_or(Error::IdNotFound {
                id: self.id.to_string(),
            })
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

pub struct UpdateAcme {
    pub id: ShortId,
    pub config: AcmeConfig,
}

#[async_trait::async_trait]
impl RpcMethod for UpdateAcme {
    type Output = ();

    async fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        let entry = state.acmes.update(self.id, self.config)?;
        state.storage.save_acme(&entry).await;
        state.update_acmes().await;
        Ok(())
    }
}

pub struct DeleteAcme {
    pub id: ShortId,
}

#[async_trait::async_trait]
impl RpcMethod for DeleteAcme {
    type Output = ();

    async fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        state.acmes.delete(self.id)?;
        state.update_acmes().await;
        state.storage.delete_acme(self.id).await;
        Ok(())
    }
}
