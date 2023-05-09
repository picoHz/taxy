use super::AppState;
use crate::{config::AppConfig, server::rpc::config::*};
use warp::{Rejection, Reply};

/// Get the application configuration.
#[utoipa::path(
    get,
    path = "/api/config",
    responses(
        (status = 200, body = AppConfig),
        (status = 401),
    ),
    security(
        ("authorization"=[])
    )
)]
pub async fn get(state: AppState) -> Result<impl Reply, Rejection> {
    Ok(warp::reply::json(&state.call(GetConfig).await?))
}

/// Update the application configuration.
#[utoipa::path(
    put,
    path = "/api/config",
    request_body = AppConfig,
    responses(
        (status = 200),
        (status = 400, body = Error),
        (status = 401),
    ),
    security(
        ("authorization"=[])
    )
)]
pub async fn put(state: AppState, config: AppConfig) -> Result<impl Reply, Rejection> {
    Ok(warp::reply::json(&state.call(SetConfig { config }).await?))
}
