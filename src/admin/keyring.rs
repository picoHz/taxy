use super::{log::LogQuery, AppState};
use crate::{
    command::ServerCommand,
    error::Error,
    keyring::{
        acme::{AcmeEntry, AcmeRequest},
        certs::{Cert, SelfSignedCertRequest},
        KeyringItem,
    },
};
use std::{io::Read, sync::Arc};
use tokio_stream::StreamExt;
use utoipa::ToSchema;
use warp::{multipart::FormData, Buf, Rejection, Reply};

/// List keyring items.
#[utoipa::path(
    get,
    path = "/api/keyring",
    responses(
        (status = 200, body = [KeyringInfo]),
        (status = 401),
    ),
    security(
        ("authorization"=[])
    )
)]
pub async fn list(state: AppState) -> Result<impl Reply, Rejection> {
    let data = state.data.lock().await;
    Ok(warp::reply::json(&data.keyring_items))
}

/// Generate a self-signed certificate.
#[utoipa::path(
    post,
    path = "/api/keyring/self_signed",
    request_body = SelfSignedCertRequest,
    responses(
        (status = 200),
        (status = 400, body = Error),
        (status = 401),
    ),
    security(
        ("authorization"=[])
    )
)]
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

#[derive(ToSchema)]
#[allow(dead_code)]
pub struct CertPostBody {
    #[schema(format = Binary)]
    chain: String,
    #[schema(format = Binary)]
    key: String,
}

/// Upload a certificate and key pair.
#[utoipa::path(
    post,
    path = "/api/keyring/upload",
    request_body(content = CertPostBody, content_type = "multipart/form-data"),
    responses(
        (status = 200),
        (status = 400, body = Error),
        (status = 401),
    ),
    security(
        ("authorization"=[])
    )
)]
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

/// Register an ACME configuration.
#[utoipa::path(
    post,
    path = "/api/keyring/acme",
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
pub async fn acme(state: AppState, request: AcmeRequest) -> Result<impl Reply, Rejection> {
    let item = KeyringItem::Acme(Arc::new(AcmeEntry::new(request).await?));
    let _ = state
        .sender
        .send(ServerCommand::AddKeyringItem { item })
        .await;
    Ok(warp::reply::reply())
}

/// Delete a keyring item.
#[utoipa::path(
    delete,
    path = "/api/keyring/{id}",
    params(
        ("id" = String, Path, description = "Certification ID")
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
    if !state
        .data
        .lock()
        .await
        .keyring_items
        .iter()
        .any(|c| c.id() == id)
    {
        return Err(warp::reject::custom(Error::KeyringItemNotFound { id }));
    }
    let _ = state
        .sender
        .send(ServerCommand::DeleteKeyringItem { id })
        .await;
    Ok(warp::reply::reply())
}

/// Get log.
#[utoipa::path(
    get,
    path = "/api/keyring/{id}/log",
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
    let data = state.data.lock().await;
    if let Some(item) = data.keyring_items.iter().find(|c| c.id() == id) {
        let log = data.log.clone();
        let id = item.id().to_string();
        std::mem::drop(data);
        let rows = log.fetch_system_log(&id, query.since, query.until).await?;
        Ok(warp::reply::json(&rows))
    } else {
        Err(warp::reject::custom(Error::KeyringItemNotFound { id }))
    }
}
