use super::RpcMethod;
use crate::server::state::ServerState;
use taxy_api::error::Error;

pub struct VerifyAccount {
    pub username: String,
    pub password: String,
}

#[async_trait::async_trait]
impl RpcMethod for VerifyAccount {
    type Output = ();

    async fn call(self, state: &mut ServerState) -> Result<Self::Output, Error> {
        state
            .storage
            .verify_account(&self.username, &self.password)
            .await
    }
}
