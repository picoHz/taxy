use super::{AppError, AppState};
use crate::server::rpc::proxies::{
    AddProxy, DeleteProxy, GetProxy, GetProxyList, GetProxyStatus, UpdateProxy,
};
use axum::{
    extract::{Path, State},
    Json,
};
use taxy_api::{
    id::ShortId,
    proxy::{Proxy, ProxyEntry, ProxyStatus},
};

pub async fn list(State(state): State<AppState>) -> Result<Json<Box<Vec<ProxyEntry>>>, AppError> {
    Ok(Json(state.call(GetProxyList).await?))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<ShortId>,
) -> Result<Json<Box<ProxyEntry>>, AppError> {
    Ok(Json(state.call(GetProxy { id }).await?))
}

pub async fn status(
    State(state): State<AppState>,
    Path(id): Path<ShortId>,
) -> Result<Json<Box<ProxyStatus>>, AppError> {
    Ok(Json(state.call(GetProxyStatus { id }).await?))
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<ShortId>,
) -> Result<Json<Box<()>>, AppError> {
    Ok(Json(state.call(DeleteProxy { id }).await?))
}

pub async fn add(
    State(state): State<AppState>,
    Json(entry): Json<Proxy>,
) -> Result<Json<Box<()>>, AppError> {
    Ok(Json(state.call(AddProxy { entry }).await?))
}

pub async fn put(
    State(state): State<AppState>,
    Path(id): Path<ShortId>,
    Json(entry): Json<Proxy>,
) -> Result<Json<Box<()>>, AppError> {
    let entry = (id, entry).into();
    Ok(Json(state.call(UpdateProxy { entry }).await?))
}
