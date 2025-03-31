use super::{AppError, AppState};
use crate::server::rpc::ports::{
    AddPort, DeletePort, GetNetworkInterfaceList, GetPort, GetPortList, GetPortStatus, ResetPort,
    UpdatePort,
};
use axum::{
    extract::{Path, State},
    Json,
};
use taxy_api::{
    id::ShortId,
    port::{NetworkInterface, Port, PortEntry, PortStatus},
};

pub async fn list(State(state): State<AppState>) -> Result<Json<Box<Vec<PortEntry>>>, AppError> {
    Ok(Json(state.call(GetPortList).await?))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<ShortId>,
) -> Result<Json<Box<PortEntry>>, AppError> {
    Ok(Json(state.call(GetPort { id }).await?))
}

pub async fn status(
    State(state): State<AppState>,
    Path(id): Path<ShortId>,
) -> Result<Json<Box<PortStatus>>, AppError> {
    Ok(Json(state.call(GetPortStatus { id }).await?))
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<ShortId>,
) -> Result<Json<Box<()>>, AppError> {
    Ok(Json(state.call(DeletePort { id }).await?))
}

pub async fn add(
    State(state): State<AppState>,
    Json(entry): Json<Port>,
) -> Result<Json<Box<()>>, AppError> {
    Ok(Json(state.call(AddPort { entry }).await?))
}

pub async fn put(
    State(state): State<AppState>,
    Path(id): Path<ShortId>,
    Json(entry): Json<Port>,
) -> Result<Json<Box<()>>, AppError> {
    let entry = (id, entry).into();
    Ok(Json(state.call(UpdatePort { entry }).await?))
}

pub async fn reset(
    State(state): State<AppState>,
    Path(id): Path<ShortId>,
) -> Result<Json<Box<()>>, AppError> {
    Ok(Json(state.call(ResetPort { id }).await?))
}

pub async fn interfaces(
    State(state): State<AppState>,
) -> Result<Json<Box<Vec<NetworkInterface>>>, AppError> {
    Ok(Json(state.call(GetNetworkInterfaceList).await?))
}
