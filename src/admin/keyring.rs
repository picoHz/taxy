use super::{log::LogQuery, AppState};
use crate::{
    error::Error,
    keyring::{
        acme::{AcmeEntry, AcmeRequest},
        certs::{Cert, SelfSignedCertRequest},
        KeyringItem,
    },
    server::rpc::keyring::*,
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
    Ok(warp::reply::json(&state.call(GetKeyringItemList).await?))
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
    Ok(warp::reply::json(
        &state.call(AddKeyringItem { item }).await?,
    ))
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
    Ok(warp::reply::json(
        &state.call(AddKeyringItem { item }).await?,
    ))
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
    Ok(warp::reply::json(
        &state.call(AddKeyringItem { item }).await?,
    ))
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
    Ok(warp::reply::json(
        &state.call(DeleteKeyringItem { id }).await?,
    ))
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
    let log = state.data.lock().await.log.clone();
    let rows = log
        .fetch_system_log(&id, query.since, query.until, query.limit)
        .await?;
    Ok(warp::reply::json(&rows))
}
