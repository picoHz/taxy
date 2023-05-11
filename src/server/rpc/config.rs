use super::RpcMethod;
use crate::{config::AppConfig, error::Error, server::state::ServerState};

pub struct GetConfig;

impl RpcMethod for GetConfig {
    const NAME: &'static str = "get_config";
    type Output = AppConfig;

    fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        Ok(state.config().clone())
    }
}

pub struct SetConfig {
    pub config: AppConfig,
}

impl RpcMethod for SetConfig {
    const NAME: &'static str = "set_config";
    type Output = ();

    fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        state.set_config(self.config);
        Ok(())
    }
}
