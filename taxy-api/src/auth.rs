use serde_derive::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, ToSchema)]
pub struct LoginRequest {
    #[schema(example = "admin")]
    pub username: String,
    #[schema(inline)]
    #[serde(flatten)]
    pub method: LoginMethod,
}

#[derive(Deserialize, Serialize, ToSchema)]
#[serde(tag = "method", rename_all = "snake_case")]
pub enum LoginMethod {
    Password {
        #[schema(example = "passw0rd")]
        password: String,
    },
    Totp {
        #[schema(example = "234567")]
        token: String,
    },
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum LoginResponse {
    Success,
    TotpRequired,
}
