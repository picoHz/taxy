use super::RpcMethod;
use crate::{
    error::Error,
    keyring::acme::{AcmeEntry, AcmeInfo},
    server::state::ServerState,
};

pub struct GetAcmeList;

impl RpcMethod for GetAcmeList {
    const NAME: &'static str = "get_acme_list";
    type Output = Vec<AcmeInfo>;

    fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        Ok(state.get_acme_list())
    }
}

pub struct AddAcme {
    pub item: AcmeEntry,
}

impl RpcMethod for AddAcme {
    const NAME: &'static str = "add_acme";
    type Output = ();

    fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        state.add_acme(self.item)
    }
}

pub struct DeleteAcme {
    pub id: String,
}

impl RpcMethod for DeleteAcme {
    const NAME: &'static str = "delete_acme";
    type Output = ();

    fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        state.delete_acme(&self.id)
    }
}
