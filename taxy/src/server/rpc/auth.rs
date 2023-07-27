use super::RpcMethod;
use crate::server::state::ServerState;
use taxy_api::{
    auth::{LoginRequest, LoginResponse},
    error::Error,
};

pub struct VerifyAccount {
    pub request: LoginRequest,
}

#[async_trait::async_trait]
impl RpcMethod for VerifyAccount {
    type Output = LoginResponse;

    async fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        state.storage.verify_account(self.request).await
    }
}
