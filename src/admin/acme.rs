use super::{log::LogQuery, AppState};
use crate::{
    keyring::acme::{AcmeEntry, AcmeRequest},
    server::rpc::acme::*,
};
use warp::{Rejection, Reply};

/// List ACME configurations.
#[utoipa::path(
    get,
    path = "/api/acme",
    responses(
        (status = 200, body = [AcmeInfo]),
        (status = 401),
    ),
    security(
        ("authorization"=[])
    )
)]
pub async fn list(state: AppState) -> Result<impl Reply, Rejection> {
    Ok(warp::reply::json(&state.call(GetAcmeList).await?))
}

/// Register an ACME configuration.
#[utoipa::path(
    post,
    path = "/api/acme",
    request_body = AcmeRequest,
    responses(
        (status = 200),
        (status = 400, body = Error),
        (status = 401),
    ),
    security(
        ("authorization"=[])
    )
)]
pub async fn add(state: AppState, request: AcmeRequest) -> Result<impl Reply, Rejection> {
    let item = AcmeEntry::new(request).await?;
    Ok(warp::reply::json(&state.call(AddAcme { item }).await?))
}

/// Delete an ACME configuration.
#[utoipa::path(
    delete,
    path = "/api/acme/{id}",
    params(
        ("id" = String, Path, description = "ACME ID")
    ),
    responses(
        (status = 200),
        (status = 404),
        (status = 401),
    ),
    security(
        ("authorization"=[])
    )
)]
pub async fn delete(state: AppState, id: String) -> Result<impl Reply, Rejection> {
    Ok(warp::reply::json(&state.call(DeleteAcme { id }).await?))
}

/// Get log.
#[utoipa::path(
    get,
    path = "/api/acme/{id}/log",
    params(
        ("id" = String, Path, description = "Item ID"),
        LogQuery
    ),
    responses(
        (status = 200, body = Vec<SystemLogRow>),
        (status = 408),
        (status = 404),
        (status = 401),
    ),
    security(
        ("authorization"=[])
    )
)]
pub async fn log(state: AppState, id: String, query: LogQuery) -> Result<impl Reply, Rejection> {
    let log = state.data.lock().await.log.clone();
    let rows = log
        .fetch_system_log(&id, query.since, query.until, query.limit)
        .await?;
    Ok(warp::reply::json(&rows))
}
