use super::AppState;
use crate::{command::ServerCommand, config::AppConfig};
use warp::{Rejection, Reply};

/// Get the application configuration.
#[utoipa::path(
    get,
    path = "/api/config",
    responses(
        (status = 200, body = AppConfig)
    )
)]
pub async fn get(state: AppState) -> Result<impl Reply, Rejection> {
    let data = state.data.lock().await;
    Ok(warp::reply::json(&data.config))
}

/// Update the application configuration.
#[utoipa::path(
    put,
    path = "/api/config",
    request_body = AppConfig,
    responses(
        (status = 200),
        (status = 400, body = Error),
    )
)]
pub async fn put(state: AppState, config: AppConfig) -> Result<impl Reply, Rejection> {
    let _ = state
        .sender
        .send(ServerCommand::SetAppConfig { config })
        .await;
    Ok(warp::reply::reply())
}
