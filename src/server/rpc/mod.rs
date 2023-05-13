use super::state::ServerState;
use crate::error::Error;
use std::any::Any;

pub mod acme;
pub mod config;
pub mod keyring;
pub mod ports;

pub type RpcCallbackFunc =
    Box<dyn Fn(&mut ServerState, Box<dyn Any>) -> Result<Box<dyn Any + Send + Sync>, Error> + Send>;

pub trait RpcMethod: Any + Send + Sync {
    const NAME: &'static str;
    type Output: Any + Send + Sync;
    fn call(self, state: &mut ServerState) -> Result<Self::Output, Error>;
}

pub struct RpcCallback {
    pub id: usize,
    pub result: Result<Box<dyn Any + Send + Sync>, Error>,
}
