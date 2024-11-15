use super::{AppError, AppState};
use crate::server::rpc::config::{GetConfig, SetConfig};
use axum::{extract::State, Json};
use taxy_api::app::AppConfig;

pub async fn get(State(state): State<AppState>) -> Result<Json<Box<AppConfig>>, AppError> {
    Ok(Json(state.call(GetConfig).await?))
}

pub async fn put(
    State(state): State<AppState>,
    Json(config): Json<AppConfig>,
) -> Result<Json<Box<()>>, AppError> {
    Ok(Json(state.call(SetConfig { config }).await?))
}
