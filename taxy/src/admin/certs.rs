use super::{AppError, AppState};
use crate::{
    certs::Cert,
    server::rpc::certs::{AddCert, DeleteCert, DownloadCert, GetCert, GetCertList},
};
use axum::{
    extract::{Multipart, Path, Query, State},
    http::HeaderMap,
    response::IntoResponse,
    Json,
};
use std::{ops::Deref, sync::Arc};
use taxy_api::{
    cert::{CertInfo, SelfSignedCertRequest, UploadQuery},
    id::ShortId,
};

pub async fn list(State(state): State<AppState>) -> Result<Json<Box<Vec<CertInfo>>>, AppError> {
    Ok(Json(state.call(GetCertList).await?))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<ShortId>,
) -> Result<Json<Box<CertInfo>>, AppError> {
    let cert = state.call(GetCert { id }).await?;
    Ok(Json(Box::new(cert.info())))
}

pub async fn self_sign(
    State(state): State<AppState>,
    Json(request): Json<SelfSignedCertRequest>,
) -> Result<Json<Box<()>>, AppError> {
    let cert = if let Some(ca_cert) = request.ca_cert {
        let ca = state.call(GetCert { id: ca_cert }).await?;
        Cert::new_self_signed(&request.san, &ca)?
    } else {
        let ca = Arc::new(Cert::new_ca()?);
        state.call(AddCert { cert: ca.clone() }).await?;
        Cert::new_self_signed(&request.san, &ca)?
    };
    let cert = Arc::new(cert);
    Ok(Json(state.call(AddCert { cert }).await?))
}

pub async fn upload(
    State(state): State<AppState>,
    Query(query): Query<UploadQuery>,
    mut multipart: Multipart,
) -> Result<Json<Box<()>>, AppError> {
    let mut chain = Vec::new();
    let mut key = Vec::new();
    while let Ok(Some(field)) = multipart.next_field().await {
        if field.name() == Some("chain") {
            if let Ok(buf) = field.bytes().await {
                chain = buf.to_vec();
            }
        } else if field.name() == Some("key") {
            if let Ok(buf) = field.bytes().await {
                key = buf.to_vec();
            }
        }
    }

    let key = if key.is_empty() { None } else { Some(key) };
    let cert = Arc::new(Cert::new(query.kind, chain, key)?);
    Ok(Json(state.call(AddCert { cert }).await?))
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<ShortId>,
) -> Result<Json<Box<()>>, AppError> {
    Ok(Json(state.call(DeleteCert { id }).await?))
}

pub async fn download(
    State(state): State<AppState>,
    Path(id): Path<ShortId>,
) -> Result<impl IntoResponse, AppError> {
    let file = state.call(DownloadCert { id }).await?;
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/gzip".parse().unwrap());
    headers.insert(
        "Content-Disposition",
        format!("attachment; filename=\"{}.tar.gz\"", id)
            .parse()
            .unwrap(),
    );
    Ok((headers, file.deref().clone()))
}
