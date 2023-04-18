use crate::{command::ServerCommand, config::AppConfig};

use super::AppState;
use warp::{Rejection, Reply};

pub async fn get(state: AppState) -> Result<impl Reply, Rejection> {
    let data = state.data.lock().await;
    Ok(warp::reply::json(&data.config))
}

pub async fn put(state: AppState, config: AppConfig) -> Result<impl Reply, Rejection> {
    let _ = state
        .sender
        .send(ServerCommand::SetAppConfig { config })
        .await;
    Ok(warp::reply::reply())
}
