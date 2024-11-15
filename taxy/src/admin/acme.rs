use super::{AppError, AppState};
use crate::server::rpc::acme::{AddAcme, DeleteAcme, GetAcme, GetAcmeList, UpdateAcme};
use axum::{
    extract::{Path, State},
    Json,
};
use taxy_api::{
    acme::{AcmeConfig, AcmeInfo, AcmeRequest},
    id::ShortId,
};

pub async fn list(State(state): State<AppState>) -> Result<Json<Box<Vec<AcmeInfo>>>, AppError> {
    Ok(Json(state.call(GetAcmeList).await?))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<ShortId>,
) -> Result<Json<Box<AcmeInfo>>, AppError> {
    Ok(Json(state.call(GetAcme { id }).await?))
}

pub async fn add(
    State(state): State<AppState>,
    Json(request): Json<AcmeRequest>,
) -> Result<Json<Box<()>>, AppError> {
    Ok(Json(state.call(AddAcme { request }).await?))
}

pub async fn put(
    State(state): State<AppState>,
    Path(id): Path<ShortId>,
    Json(config): Json<AcmeConfig>,
) -> Result<Json<Box<()>>, AppError> {
    Ok(Json(state.call(UpdateAcme { id, config }).await?))
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<ShortId>,
) -> Result<Json<Box<()>>, AppError> {
    Ok(Json(state.call(DeleteAcme { id }).await?))
}
