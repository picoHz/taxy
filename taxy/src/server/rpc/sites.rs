use super::RpcMethod;
use crate::server::state::ServerState;
use taxy_api::error::Error;
use taxy_api::site::{Site, SiteEntry};

pub struct GetSiteList;

#[async_trait::async_trait]
impl RpcMethod for GetSiteList {
    type Output = Vec<SiteEntry>;

    async fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        Ok(state.sites.entries().cloned().collect())
    }
}

pub struct GetSite {
    pub id: String,
}

#[async_trait::async_trait]
impl RpcMethod for GetSite {
    type Output = SiteEntry;

    async fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        state
            .sites
            .get(&self.id)
            .cloned()
            .ok_or(Error::IdNotFound { id: self.id })
    }
}

pub struct DeleteSite {
    pub id: String,
}

#[async_trait::async_trait]
impl RpcMethod for DeleteSite {
    type Output = ();

    async fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        state.sites.delete(&self.id)?;
        state.update_sites().await;
        Ok(())
    }
}

pub struct AddSite {
    pub entry: Site,
}

#[async_trait::async_trait]
impl RpcMethod for AddSite {
    type Output = ();

    async fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        state.sites.add((state.generate_id(), self.entry).into())?;
        state.update_sites().await;
        Ok(())
    }
}

pub struct UpdateSite {
    pub entry: SiteEntry,
}

#[async_trait::async_trait]
impl RpcMethod for UpdateSite {
    type Output = ();

    async fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        state.sites.update(self.entry)?;
        state.update_sites().await;
        Ok(())
    }
}
