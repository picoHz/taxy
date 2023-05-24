use super::RpcMethod;
use crate::{config::site::SiteEntry, error::Error, server::state::ServerState};
pub struct GetSiteList;

#[async_trait::async_trait]
impl RpcMethod for GetSiteList {
    type Output = Vec<SiteEntry>;

    async fn call(&self, state: &mut ServerState) -> Result<Self::Output, Error> {
        Ok(state.get_site_list())
    }
}

pub struct DeleteSite {
    pub id: String,
}

#[async_trait::async_trait]
impl RpcMethod for DeleteSite {
    type Output = ();

    async fn call(&self, state: &mut ServerState) -> Result<Self::Output, Error> {
        state.delete_site(&self.id)
    }
}

pub struct AddSite {
    pub entry: SiteEntry,
}

#[async_trait::async_trait]
impl RpcMethod for AddSite {
    type Output = ();

    async fn call(&self, state: &mut ServerState) -> Result<Self::Output, Error> {
        state.add_site(self.entry.clone())
    }
}

pub struct UpdateSite {
    pub entry: SiteEntry,
}

#[async_trait::async_trait]
impl RpcMethod for UpdateSite {
    type Output = ();

    async fn call(&self, state: &mut ServerState) -> Result<Self::Output, Error> {
        state.update_site(self.entry.clone())
    }
}
