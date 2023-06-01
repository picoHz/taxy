use super::state::ServerState;
use std::any::Any;
use taxy_api::error::Error;

pub mod acme;
pub mod config;
pub mod ports;
pub mod server_certs;
pub mod sites;

#[async_trait::async_trait]
pub trait RpcMethod: Any + Send + Sync {
    type Output: Any + Send + Sync;
    async fn call(self, state: &mut ServerState) -> Result<Self::Output, Error>;
}

pub struct RpcWrapper<T: RpcMethod> {
    inner: Option<T>,
}

impl<T> RpcWrapper<T>
where
    T: RpcMethod,
{
    pub fn new(inner: T) -> Self {
        Self { inner: Some(inner) }
    }
}

#[async_trait::async_trait]
impl<T> ErasedRpcMethod for RpcWrapper<T>
where
    T: RpcMethod,
{
    async fn call(&mut self, state: &mut ServerState) -> Result<Box<dyn Any + Send + Sync>, Error> {
        let this = self.inner.take().ok_or(Error::RpcError)?;
        <T as RpcMethod>::call(this, state)
            .await
            .map(|r| Box::new(r) as Box<dyn Any + Send + Sync>)
    }
}

#[async_trait::async_trait]
pub trait ErasedRpcMethod: Any + Send + Sync {
    async fn call(&mut self, state: &mut ServerState) -> Result<Box<dyn Any + Send + Sync>, Error>;
}

pub struct RpcCallback {
    pub id: usize,
    pub result: Result<Box<dyn Any + Send + Sync>, Error>,
}
