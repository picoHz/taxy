use super::AppState;
use crate::{
    command::ServerCommand,
    error::Error,
    keyring::{
        certs::{Cert, SelfSignedCertRequest},
        KeyringItem,
    },
};
use std::{io::Read, sync::Arc};
use tokio_stream::StreamExt;
use warp::{multipart::FormData, Buf, Rejection, Reply};

pub async fn list(state: AppState) -> Result<impl Reply, Rejection> {
    let data = state.data.lock().await;
    Ok(warp::reply::json(&data.keyring_items))
}

pub async fn self_signed(
    state: AppState,
    request: SelfSignedCertRequest,
) -> Result<impl Reply, Rejection> {
    let item = KeyringItem::ServerCert(Arc::new(Cert::new_self_signed(&request)?));
    let _ = state
        .sender
        .send(ServerCommand::AddKeyringItem { item })
        .await;
    Ok(warp::reply::reply())
}

pub async fn upload(state: AppState, mut form: FormData) -> Result<impl Reply, Rejection> {
    let mut chain = Vec::new();
    let mut key = Vec::new();
    while let Some(part) = form.next().await {
        if let Ok(mut part) = part {
            if part.name() == "chain" {
                if let Some(Ok(buf)) = part.data().await {
                    buf.reader()
                        .read_to_end(&mut chain)
                        .map_err(|_| Error::FailedToReadCertificate)?;
                }
            } else if part.name() == "key" {
                if let Some(Ok(buf)) = part.data().await {
                    buf.reader()
                        .read_to_end(&mut key)
                        .map_err(|_| Error::FailedToReadPrivateKey)?;
                }
            }
        }
    }

    let item = KeyringItem::ServerCert(Arc::new(Cert::new(chain, key)?));
    if state
        .data
        .lock()
        .await
        .keyring_items
        .iter()
        .any(|c| c.id() == item.id())
    {
        return Err(warp::reject::custom(Error::CertAlreadyExists {
            id: item.id().to_string(),
        }));
    }
    let _ = state
        .sender
        .send(ServerCommand::AddKeyringItem { item })
        .await;
    Ok(warp::reply::reply())
}

pub async fn delete(state: AppState, id: String) -> Result<impl Reply, Rejection> {
    if !state
        .data
        .lock()
        .await
        .keyring_items
        .iter()
        .any(|c| c.id() == id)
    {
        return Err(warp::reject::custom(Error::CertNotFound { id }));
    }
    let _ = state
        .sender
        .send(ServerCommand::DeleteKeyringItem { id })
        .await;
    Ok(warp::reply::reply())
}
