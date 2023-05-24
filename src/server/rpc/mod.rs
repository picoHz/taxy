use super::state::ServerState;
use crate::error::Error;
use std::any::Any;

pub mod acme;
pub mod config;
pub mod ports;
pub mod server_certs;
pub mod sites;

#[async_trait::async_trait]
pub trait RpcMethod: Any + Send + Sync {
    type Output: Any + Send + Sync;
    async fn call(&self, state: &mut ServerState) -> Result<Self::Output, Error>;
}

#[async_trait::async_trait]
pub trait ErasedRpcMethod: Any + Send + Sync {
    async fn call(&self, state: &mut ServerState) -> Result<Box<dyn Any + Send + Sync>, Error>;
}

#[async_trait::async_trait]
impl<T> ErasedRpcMethod for T where T: RpcMethod  {
    async fn call(&self, state: &mut ServerState) -> Result<Box<dyn Any + Send + Sync>, Error> {
        <Self as RpcMethod>::call(self, state).await.map(|r| Box::new(r) as Box<dyn Any + Send + Sync>)
    }
}

pub struct RpcCallback {
    pub id: usize,
    pub result: Result<Box<dyn Any + Send + Sync>, Error>,
}
