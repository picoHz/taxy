use super::RpcMethod;
use crate::server::state::ServerState;
use taxy_api::error::Error;
use taxy_api::port::{PortEntry, PortStatus};

pub struct GetPortList;

#[async_trait::async_trait]
impl RpcMethod for GetPortList {
    type Output = Vec<PortEntry>;

    async fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        Ok(state.get_port_list())
    }
}

pub struct GetPortStatus {
    pub id: String,
}

#[async_trait::async_trait]
impl RpcMethod for GetPortStatus {
    type Output = PortStatus;

    async fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        state.get_port_status(&self.id)
    }
}

pub struct DeletePort {
    pub id: String,
}

#[async_trait::async_trait]
impl RpcMethod for DeletePort {
    type Output = ();

    async fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        state.delete_port(&self.id).await
    }
}

pub struct AddPort {
    pub entry: PortEntry,
}

#[async_trait::async_trait]
impl RpcMethod for AddPort {
    type Output = ();

    async fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        state.add_port(self.entry).await
    }
}

pub struct UpdatePort {
    pub entry: PortEntry,
}

#[async_trait::async_trait]
impl RpcMethod for UpdatePort {
    type Output = ();

    async fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        state.update_port(self.entry).await
    }
}

pub struct ResetPort {
    pub id: String,
}

#[async_trait::async_trait]
impl RpcMethod for ResetPort {
    type Output = ();

    async fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        state.reset_port(&self.id)
    }
}
