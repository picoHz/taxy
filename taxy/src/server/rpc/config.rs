use super::RpcMethod;
use crate::{config::AppConfig, error::Error, server::state::ServerState};

pub struct GetConfig;

#[async_trait::async_trait]
impl RpcMethod for GetConfig {
    type Output = AppConfig;

    async fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        Ok(state.config().clone())
    }
}

pub struct SetConfig {
    pub config: AppConfig,
}

#[async_trait::async_trait]
impl RpcMethod for SetConfig {
    type Output = ();

    async fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        state.set_config(self.config).await
    }
}
