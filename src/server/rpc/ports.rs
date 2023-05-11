use super::RpcMethod;
use crate::{config::port::PortEntry, error::Error, proxy::PortStatus, server::state::ServerState};

pub struct GetPortList;

impl RpcMethod for GetPortList {
    const NAME: &'static str = "get_port_list";
    type Output = Vec<PortEntry>;

    fn call(&self, state: &mut ServerState) -> Result<Self::Output, Error> {
        Ok(state.get_port_list())
    }
}

pub struct GetPortStatus {
    pub id: String,
}

impl RpcMethod for GetPortStatus {
    const NAME: &'static str = "get_port_status";
    type Output = PortStatus;

    fn call(&self, state: &mut ServerState) -> Result<Self::Output, Error> {
        state.get_port_status(&self.id)
    }
}

pub struct DeletePort {
    pub id: String,
}

impl RpcMethod for DeletePort {
    const NAME: &'static str = "delete_port";
    type Output = ();

    fn call(&self, state: &mut ServerState) -> Result<Self::Output, Error> {
        state.delete_port(&self.id)
    }
}

pub struct AddPort {
    pub entry: PortEntry,
}

impl RpcMethod for AddPort {
    const NAME: &'static str = "add_port";
    type Output = ();

    fn call(&self, state: &mut ServerState) -> Result<Self::Output, Error> {
        state.add_port(self.entry.clone())
    }
}
