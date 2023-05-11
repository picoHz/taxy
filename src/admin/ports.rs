use super::AppState;
use crate::{
    admin::log::LogQuery,
    config::port::{PortEntry, PortEntryRequest},
    server::rpc::ports::*,
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
    Ok(warp::reply::json(
        &state
            .call(AddPort {
                entry: entry.into(),
            })
            .await?,
    ))
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
    let mut entry: PortEntry = entry.into();
    entry.id = id;
    Ok(warp::reply::json(
        &state
            .call(UpdatePort {
                entry: entry.into(),
            })
            .await?,
    ))
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
    let log = state.data.lock().await.log.clone();
    let rows = log
        .fetch_system_log(&id, query.since, query.until, query.limit)
        .await?;
    Ok(warp::reply::json(&rows))
}
