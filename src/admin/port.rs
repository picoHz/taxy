use super::AppState;
use crate::{
    command::ServerCommand,
    config::port::{PortEntry, PortEntryRequest},
    error::Error,
    proxy::PortContext,
};
use warp::{Rejection, Reply};

/// Get the list of port configurations.
#[utoipa::path(
    get,
    path = "/api/ports",
    responses(
        (status = 200, body = [PortEntry])
    )
)]
pub async fn list(state: AppState) -> Result<impl Reply, Rejection> {
    let data = state.data.lock().await;
    Ok(warp::reply::json(&data.entries))
}

/// Get the status of a port.
#[utoipa::path(
    get,
    path = "/api/ports/{id}/status",
    params(
        ("id" = String, Path, description = "Port configuration id")
    ),
    responses(
        (status = 200, body = PortStatus),
        (status = 404)
    )
)]
pub async fn status(state: AppState, id: String) -> Result<impl Reply, Rejection> {
    let data = state.data.lock().await;
    if let Some(status) = data.status.get(&id) {
        Ok(warp::reply::json(&status))
    } else {
        Err(warp::reject::custom(Error::IdNotFound { id }))
    }
}

/// Delete a port configuration.
#[utoipa::path(
    delete,
    path = "/api/ports/{id}",
    params(
        ("id" = String, Path, description = "Port configuration id")
    ),
    responses(
        (status = 200),
        (status = 404),
    )
)]
pub async fn delete(state: AppState, id: String) -> Result<impl Reply, Rejection> {
    if state.data.lock().await.entries.iter().all(|e| e.id != id) {
        return Err(warp::reject::custom(Error::IdNotFound { id }));
    }
    let _ = state.sender.send(ServerCommand::DeletePort { id }).await;
    Ok(warp::reply::reply())
}

/// Create a new port configuration.
#[utoipa::path(
    post,
    path = "/api/ports",
    request_body = PortEntry,
    responses(
        (status = 200),
        (status = 400, body = Error),
    )
)]
pub async fn post(state: AppState, entry: PortEntryRequest) -> Result<impl Reply, Rejection> {
    let data = state.data.lock().await;
    let mut ctx = PortContext::new(entry.into())?;
    ctx.prepare(&data.config).await?;
    let _ = state.sender.send(ServerCommand::SetPort { ctx }).await;
    Ok(warp::reply::reply())
}

/// Update or rename a port configuration.
#[utoipa::path(
    put,
    path = "/api/ports/{id}",
    params(
        ("id" = String, Path, description = "Port configuration name")
    ),
    request_body = PortEntry,
    responses(
        (status = 200),
        (status = 404),
        (status = 400, body = Error),
    )
)]
pub async fn put(
    state: AppState,
    entry: PortEntryRequest,
    id: String,
) -> Result<impl Reply, Rejection> {
    let data = state.data.lock().await;
    let mut entry: PortEntry = entry.into();
    entry.id = id;
    let mut ctx = PortContext::new(entry)?;
    ctx.prepare(&data.config).await?;
    let _ = state.sender.send(ServerCommand::SetPort { ctx }).await;
    Ok(warp::reply::reply())
}
