use super::AppState;
use crate::{command::ServerCommand, config::port::PortEntry, error::Error, proxy::PortContext};
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
    path = "/api/ports/{name}/status",
    params(
        ("name" = String, Path, description = "Port configuration name")
    ),
    responses(
        (status = 200, body = PortStatus),
        (status = 404)
    )
)]
pub async fn status(state: AppState, name: String) -> Result<impl Reply, Rejection> {
    let name = percent_encoding::percent_decode_str(&name).decode_utf8_lossy();
    let data = state.data.lock().await;
    if let Some(status) = data.status.get(name.as_ref()) {
        Ok(warp::reply::json(&status))
    } else {
        Err(warp::reject::custom(Error::NameNotFound {
            name: name.to_string(),
        }))
    }
}

/// Delete a port configuration.
#[utoipa::path(
    delete,
    path = "/api/ports/{name}",
    params(
        ("name" = String, Path, description = "Port configuration name")
    ),
    responses(
        (status = 200),
        (status = 404),
    )
)]
pub async fn delete(state: AppState, name: String) -> Result<impl Reply, Rejection> {
    let name = percent_encoding::percent_decode_str(&name).decode_utf8_lossy();
    if state
        .data
        .lock()
        .await
        .entries
        .iter()
        .all(|e| e.name != name)
    {
        return Err(warp::reject::custom(Error::NameNotFound {
            name: name.to_string(),
        }));
    }
    let _ = state
        .sender
        .send(ServerCommand::DeletePort {
            name: name.to_string(),
        })
        .await;
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
pub async fn post(state: AppState, entry: PortEntry) -> Result<impl Reply, Rejection> {
    let data = state.data.lock().await;
    let name = entry.name.clone();
    if data.entries.iter().any(|e| e.name == name) {
        return Err(warp::reject::custom(Error::NameAlreadyExists { name }));
    }
    let mut ctx = PortContext::new(entry)?;
    ctx.prepare(&data.config).await?;
    let _ = state.sender.send(ServerCommand::SetPort { ctx }).await;
    Ok(warp::reply::reply())
}

/// Update or rename a port configuration.
#[utoipa::path(
    put,
    path = "/api/ports/{name}",
    params(
        ("name" = String, Path, description = "Port configuration name")
    ),
    request_body = PortEntry,
    responses(
        (status = 200),
        (status = 404),
        (status = 400, body = Error),
    )
)]
pub async fn put(state: AppState, entry: PortEntry, name: String) -> Result<impl Reply, Rejection> {
    let name = percent_encoding::percent_decode_str(&name).decode_utf8_lossy();
    if entry.name != name
        && state
            .data
            .lock()
            .await
            .entries
            .iter()
            .any(|e| e.name == entry.name)
    {
        return Err(warp::reject::custom(Error::NameAlreadyExists {
            name: entry.name,
        }));
    }
    let data = state.data.lock().await;
    let mut ctx = PortContext::new(entry)?;
    ctx.prepare(&data.config).await?;
    let _ = state.sender.send(ServerCommand::SetPort { ctx }).await;
    Ok(warp::reply::reply())
}
