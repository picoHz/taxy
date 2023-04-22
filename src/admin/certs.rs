use super::AppState;
use crate::{
    certs::{Cert, SelfSignedCertRequest},
    command::ServerCommand,
    error::Error,
};
use std::io::BufReader;
use tokio_stream::StreamExt;
use warp::{multipart::FormData, Buf, Rejection, Reply};

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
    let mut chain_reader = None;
    let mut key_reader = None;
    while let Some(part) = form.next().await {
        if let Ok(mut part) = part {
            if part.name() == "chain" {
                if let Some(Ok(buf)) = part.data().await {
                    chain_reader = Some(BufReader::new(buf.reader()));
                }
            } else if part.name() == "key" {
                if let Some(Ok(buf)) = part.data().await {
                    key_reader = Some(BufReader::new(buf.reader()));
                }
            }
        }
    }
    let mut chain_reader = chain_reader.ok_or_else(|| Error::FailedToReadCertificate)?;
    let mut key_reader = key_reader.ok_or_else(|| Error::FailedToReadPrivateKey)?;

    let cert = Cert::new(None, &mut chain_reader, &mut key_reader)?;
    let reply = warp::reply::json(&cert.info);
    let _ = state.sender.send(ServerCommand::AddCert { cert }).await;
    Ok(reply)
}
