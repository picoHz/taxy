use super::AppState;
use crate::{
    certs::{Cert, SelfSignedCertRequest},
    command::ServerCommand,
    error::Error,
};
use std::io::Read;
use tokio_stream::StreamExt;
use warp::{multipart::FormData, Buf, Rejection, Reply};

pub async fn list(state: AppState) -> Result<impl Reply, Rejection> {
    let data = state.data.lock().await;
    Ok(warp::reply::json(&data.certs))
}

pub async fn self_signed(
    state: AppState,
    request: SelfSignedCertRequest,
) -> Result<impl Reply, Rejection> {
    let cert = Cert::new_self_signed(&request)?;
    let reply = warp::reply::json(&cert.info);
    let _ = state.sender.send(ServerCommand::AddCert { cert }).await;
    Ok(reply)
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

    let cert = Cert::new(chain, key)?;
    if state
        .data
        .lock()
        .await
        .certs
        .iter()
        .any(|c| c.id == cert.info.id)
    {
        return Err(warp::reject::custom(Error::CertAlreadyExists {
            id: cert.info.id,
        }));
    }
    let reply = warp::reply::json(&cert.info);
    let _ = state.sender.send(ServerCommand::AddCert { cert }).await;
    Ok(reply)
}

pub async fn delete(state: AppState, id: String) -> Result<impl Reply, Rejection> {
    if !state.data.lock().await.certs.iter().any(|c| c.id == id) {
        return Err(warp::reject::custom(Error::CertNotFound { id }));
    }
    let _ = state.sender.send(ServerCommand::DeleteCert { id }).await;
    Ok(warp::reply::reply())
}
