use super::RpcMethod;
use crate::proxy::PortContext;
use crate::server::state::ServerState;
use network_interface::NetworkInterfaceConfig;
use taxy_api::error::Error;
use taxy_api::id::ShortId;
use taxy_api::port::{NetworkAddr, NetworkInterface, Port, PortEntry, PortStatus};

pub struct GetPortList;

#[async_trait::async_trait]
impl RpcMethod for GetPortList {
    type Output = Vec<PortEntry>;

    async fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        Ok(state.ports.entries().cloned().collect())
    }
}

pub struct GetPort {
    pub id: ShortId,
}

#[async_trait::async_trait]
impl RpcMethod for GetPort {
    type Output = PortEntry;

    async fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        state
            .ports
            .get(&self.id)
            .map(|port| port.entry().clone())
            .ok_or(Error::IdNotFound {
                id: self.id.to_string(),
            })
    }
}

pub struct GetPortStatus {
    pub id: ShortId,
}

#[async_trait::async_trait]
impl RpcMethod for GetPortStatus {
    type Output = PortStatus;

    async fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        state
            .ports
            .get(&self.id)
            .map(|port| *port.status())
            .ok_or(Error::IdNotFound {
                id: self.id.to_string(),
            })
    }
}

pub struct DeletePort {
    pub id: ShortId,
}

#[async_trait::async_trait]
impl RpcMethod for DeletePort {
    type Output = ();

    async fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        if state.ports.delete(&self.id) {
            state.update_port_statuses().await;
            Ok(())
        } else {
            Err(Error::IdNotFound {
                id: self.id.to_string(),
            })
        }
    }
}

pub struct AddPort {
    pub entry: Port,
}

#[async_trait::async_trait]
impl RpcMethod for AddPort {
    type Output = ();

    async fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        let entry: PortEntry = (state.generate_id(), self.entry).into();
        if state.ports.get(&entry.id).is_some() {
            Err(Error::IdAlreadyExists { id: entry.id })
        } else {
            if state.update_port_ctx(PortContext::new(entry)?).await {
                state.update_port_statuses().await;
            }
            Ok(())
        }
    }
}

pub struct UpdatePort {
    pub entry: PortEntry,
}

#[async_trait::async_trait]
impl RpcMethod for UpdatePort {
    type Output = ();

    async fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        if state.ports.get(&self.entry.id).is_some() {
            if state.update_port_ctx(PortContext::new(self.entry)?).await {
                state.update_port_statuses().await;
            }
            Ok(())
        } else {
            Err(Error::IdNotFound {
                id: self.entry.id.to_string(),
            })
        }
    }
}

pub struct ResetPort {
    pub id: ShortId,
}

#[async_trait::async_trait]
impl RpcMethod for ResetPort {
    type Output = ();

    async fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        if state.ports.reset(&self.id) {
            Ok(())
        } else {
            Err(Error::IdNotFound {
                id: self.id.to_string(),
            })
        }
    }
}

pub struct GetNetworkInterfaceList;

#[async_trait::async_trait]
impl RpcMethod for GetNetworkInterfaceList {
    type Output = Vec<NetworkInterface>;

    async fn call(self, _state: &mut ServerState) -> Result<Self::Output, Error> {
        Ok(network_interface::NetworkInterface::show()
            .map_err(|_| Error::FailedToListNetworkInterfaces)?
            .into_iter()
            .map(|iface| {
                let addrs = iface
                    .addr
                    .into_iter()
                    .map(|net| NetworkAddr {
                        ip: net.ip(),
                        mask: net.netmask(),
                    })
                    .collect::<Vec<_>>();
                NetworkInterface {
                    name: iface.name,
                    addrs,
                    mac: iface.mac_addr,
                }
            })
            .collect())
    }
}
