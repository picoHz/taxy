use crate::error::Error;

use super::AppState;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use warp::{Rejection, Reply};

/// Login.
#[utoipa::path(
    post,
    path = "/api/login",
    request_body = LoginRequest,
    responses(
        (status = 200),
        (status = 400),
    )
)]
pub async fn login(state: AppState, req: LoginRequest) -> Result<impl Reply, Rejection> {
    let mut data = state.data.lock().await;
    if crate::auth::verify_account(&data.app_info.config_path, &req.user, &req.password).await {
        let token = cuid2::cuid();
        data.auth_tokens.insert(token.clone());
        Ok(warp::reply::json(&LoginResult { token }))
    } else {
        Err(warp::reject::custom(Error::InvalidLoginCredentials))
    }
}

/// Logout.
#[utoipa::path(
    get,
    path = "/api/logout",
    responses(
        (status = 200),
        (status = 401),
    ),
    security(
        ("authorization"=[])
    )
)]
pub async fn logout(state: AppState, header: Option<String>) -> Result<impl Reply, Rejection> {
    if let Some(token) = get_auth_token(&header) {
        state.data.lock().await.auth_tokens.remove(token);
    }
    Ok(warp::reply::reply())
}

pub fn get_auth_token(header: &Option<String>) -> Option<&str> {
    if let Some(header) = header {
        let parts: Vec<&str> = header.split(' ').collect();
        if let [bearer, token] = &parts[..] {
            if bearer.to_lowercase() == "bearer" {
                return Some(*token);
            }
        }
    }
    None
}

#[derive(Deserialize, ToSchema)]
pub struct LoginRequest {
    #[schema(example = "admin")]
    pub user: String,
    #[schema(example = "passw0rd")]
    pub password: String,
}

#[derive(Serialize, ToSchema)]
pub struct LoginResult {
    #[schema(example = "nidhmyh9c7txiyqe53ttsxyq")]
    pub token: String,
}
