use serde_derive::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct LoginRequest {
    #[schema(example = "admin")]
    pub username: String,
    #[schema(example = "passw0rd")]
    pub password: String,
}

#[derive(Serialize, ToSchema)]
pub struct LoginResult {
    #[schema(example = "nidhmyh9c7txiyqe53ttsxyq")]
    pub token: String,
}
