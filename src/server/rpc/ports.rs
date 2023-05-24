use super::RpcMethod;
use crate::{config::port::PortEntry, error::Error, proxy::PortStatus, server::state::ServerState};

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
        state.delete_port(&self.id)
    }
}

pub struct AddPort {
    pub entry: PortEntry,
}

#[async_trait::async_trait]
impl RpcMethod for AddPort {
    type Output = ();

    async fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        state.add_port(self.entry)
    }
}

pub struct UpdatePort {
    pub entry: PortEntry,
}

#[async_trait::async_trait]
impl RpcMethod for UpdatePort {
    type Output = ();

    async fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        state.update_port(self.entry)
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
