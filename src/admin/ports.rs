use super::AppState;
use crate::{
    admin::log::LogQuery,
    command::ServerCommand,
    config::port::{PortEntry, PortEntryRequest},
    error::Error,
    proxy::PortContext,
    server::rpc::ports::{DeletePort, GetPortList, GetPortStatus},
};
use warp::{Rejection, Reply};

/// Get the list of port configurations.
#[utoipa::path(
    get,
    path = "/api/ports",
    responses(
        (status = 200, body = [PortEntry])
    ),
    responses(
        (status = 401),
    ),
    security(
        ("authorization"=[])
    )
)]
pub async fn list(state: AppState) -> Result<impl Reply, Rejection> {
    Ok(warp::reply::json(&state.call(GetPortList).await?))
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
        (status = 404),
        (status = 401),
    ),
    security(
        ("authorization"=[])
    )
)]
pub async fn status(state: AppState, id: String) -> Result<impl Reply, Rejection> {
    Ok(warp::reply::json(&state.call(GetPortStatus { id }).await?))
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
        (status = 401),
    ),
    security(
        ("authorization"=[])
    )
)]
pub async fn delete(state: AppState, id: String) -> Result<impl Reply, Rejection> {
    Ok(warp::reply::json(&state.call(DeletePort { id }).await?))
}

/// Create a new port configuration.
#[utoipa::path(
    post,
    path = "/api/ports",
    request_body = PortEntry,
    responses(
        (status = 200),
        (status = 400, body = Error),
        (status = 401),
    ),
    security(
        ("authorization"=[])
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
        (status = 401),
    ),
    security(
        ("authorization"=[])
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

/// Get log.
#[utoipa::path(
    get,
    path = "/api/ports/{id}/log",
    params(
        ("id" = String, Path, description = "Port ID"),
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
    let data = state.data.lock().await;
    if let Some(item) = data.entries.iter().find(|e| e.id == id) {
        let log = data.log.clone();
        let id = item.id.clone();
        std::mem::drop(data);
        let rows = log
            .fetch_system_log(&id, query.since, query.until, query.limit)
            .await?;
        Ok(warp::reply::json(&rows))
    } else {
        Err(warp::reject::custom(Error::KeyringItemNotFound { id }))
    }
}
