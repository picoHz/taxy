use super::AppState;
use crate::{command::ServerCommand, config::port::PortEntry, error::Error, proxy::PortContext};
use warp::{Rejection, Reply};

pub async fn list(state: AppState) -> Result<impl Reply, Rejection> {
    let data = state.data.lock().await;
    Ok(warp::reply::json(&data.entries))
}

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
