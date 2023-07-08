use super::RpcMethod;
use crate::server::state::ServerState;
use taxy_api::error::Error;
use taxy_api::site::{Proxy, ProxyEntry};

pub struct GetProxyList;

#[async_trait::async_trait]
impl RpcMethod for GetProxyList {
    type Output = Vec<ProxyEntry>;

    async fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        Ok(state.proxies.entries().cloned().collect())
    }
}

pub struct GetProxy {
    pub id: String,
}

#[async_trait::async_trait]
impl RpcMethod for GetProxy {
    type Output = ProxyEntry;

    async fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        state
            .proxies
            .get(&self.id)
            .cloned()
            .ok_or(Error::IdNotFound { id: self.id })
    }
}

pub struct DeleteProxy {
    pub id: String,
}

#[async_trait::async_trait]
impl RpcMethod for DeleteProxy {
    type Output = ();

    async fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        state.proxies.delete(&self.id)?;
        state.update_proxies().await;
        Ok(())
    }
}

pub struct AddProxy {
    pub entry: Proxy,
}

#[async_trait::async_trait]
impl RpcMethod for AddProxy {
    type Output = ();

    async fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        state
            .proxies
            .add((state.generate_id(), self.entry).into())?;
        state.update_proxies().await;
        Ok(())
    }
}

pub struct UpdateProxy {
    pub entry: ProxyEntry,
}

#[async_trait::async_trait]
impl RpcMethod for UpdateProxy {
    type Output = ();

    async fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        if state.proxies.update(self.entry)? {
            state.update_proxies().await;
        }
        Ok(())
    }
}
