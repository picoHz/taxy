use super::RpcMethod;
use crate::{config::site::SiteEntry, error::Error, server::state::ServerState};
pub struct GetSiteList;

impl RpcMethod for GetSiteList {
    const NAME: &'static str = "get_site_list";
    type Output = Vec<SiteEntry>;

    fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        Ok(state.get_site_list())
    }
}

pub struct DeleteSite {
    pub id: String,
}

impl RpcMethod for DeleteSite {
    const NAME: &'static str = "delete_port";
    type Output = ();

    fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        state.delete_site(&self.id)
    }
}

pub struct AddSite {
    pub entry: SiteEntry,
}

impl RpcMethod for AddSite {
    const NAME: &'static str = "add_port";
    type Output = ();

    fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        state.add_site(self.entry)
    }
}

pub struct UpdateSite {
    pub entry: SiteEntry,
}

impl RpcMethod for UpdateSite {
    const NAME: &'static str = "update_port";
    type Output = ();

    fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        state.update_site(self.entry)
    }
}
