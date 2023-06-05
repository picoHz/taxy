use super::RpcMethod;
use crate::server::state::ServerState;
use taxy_api::error::Error;
use taxy_api::site::SiteEntry;

pub struct GetSiteList;

#[async_trait::async_trait]
impl RpcMethod for GetSiteList {
    type Output = Vec<SiteEntry>;

    async fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        Ok(state.get_site_list())
    }
}

pub struct GetSite {
    pub id: String,
}

#[async_trait::async_trait]
impl RpcMethod for GetSite {
    type Output = SiteEntry;

    async fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        state.get_site(&self.id)
    }
}

pub struct DeleteSite {
    pub id: String,
}

#[async_trait::async_trait]
impl RpcMethod for DeleteSite {
    type Output = ();

    async fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        state.delete_site(&self.id).await
    }
}

pub struct AddSite {
    pub entry: SiteEntry,
}

#[async_trait::async_trait]
impl RpcMethod for AddSite {
    type Output = ();

    async fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        state.add_site(self.entry).await
    }
}

pub struct UpdateSite {
    pub entry: SiteEntry,
}

#[async_trait::async_trait]
impl RpcMethod for UpdateSite {
    type Output = ();

    async fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        state.update_site(self.entry).await
    }
}
