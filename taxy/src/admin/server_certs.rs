use super::{with_state, AppState};
use crate::{keyring::certs::Cert, server::rpc::server_certs::*};
use hyper::Response;
use std::{io::Read, ops::Deref};
use taxy_api::{cert::SelfSignedCertRequest, error::Error};
use tokio_stream::StreamExt;
use warp::{filters::BoxedFilter, multipart::FormData, Buf, Filter, Rejection, Reply};

pub fn api(app_state: AppState) -> BoxedFilter<(impl Reply,)> {
    let api_list = warp::get()
        .and(warp::path::end())
        .and(with_state(app_state.clone()).and_then(list));

    let api_get = warp::get().and(
        with_state(app_state.clone())
            .and(warp::path::param())
            .and(warp::path::end())
            .and_then(get),
    );

    let api_self_sign = warp::post().and(warp::path("self_sign")).and(
        with_state(app_state.clone())
            .and(warp::body::json())
            .and(warp::path::end())
            .and_then(self_sign),
    );

    let api_upload = warp::post().and(warp::path("upload")).and(
        with_state(app_state.clone())
            .and(warp::multipart::form())
            .and(warp::path::end())
            .and_then(upload),
    );

    let api_delete = warp::delete().and(
        with_state(app_state.clone())
            .and(warp::path::param())
            .and(warp::path::end())
            .and_then(delete),
    );

    let api_download = warp::get().and(
        with_state(app_state)
            .and(warp::path::param())
            .and(warp::path("download"))
            .and(warp::path::end())
            .and_then(download),
    );

    warp::path("server_certs")
        .and(
            api_delete
                .or(api_get)
                .or(api_download)
                .or(api_self_sign)
                .or(api_upload)
                .or(api_list),
        )
        .boxed()
}

/// List server certificates.
#[utoipa::path(
    get,
    path = "/api/server_certs",
    responses(
        (status = 200, body = [CertInfo]),
        (status = 401),
    ),
    security(
        ("authorization"=[])
    )
)]
pub async fn list(state: AppState) -> Result<impl Reply, Rejection> {
    Ok(warp::reply::json(&state.call(GetServerCertList).await?))
}

/// Delete a certificate.
#[utoipa::path(
    get,
    path = "/api/server_certs/{id}",
    params(
        ("id" = String, Path, description = "Certification ID")
    ),
    responses(
        (status = 200, body = [CertInfo]),
        (status = 404),
        (status = 401),
    ),
    security(
        ("authorization"=[])
    )
)]
pub async fn get(state: AppState, id: String) -> Result<impl Reply, Rejection> {
    Ok(warp::reply::json(&state.call(GetServerCert { id }).await?))
}

/// Generate a self-signed certificate.
#[utoipa::path(
    post,
    path = "/api/server_certs/self_sign",
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
pub async fn self_sign(
    state: AppState,
    request: SelfSignedCertRequest,
) -> Result<impl Reply, Rejection> {
    let cert = Cert::new_self_signed(&request)?;
    Ok(warp::reply::json(
        &state.call(AddServerCert { cert }).await?,
    ))
}

/// Upload a certificate and key pair.
#[utoipa::path(
    post,
    path = "/api/server_certs/upload",
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

    let cert = Cert::new(chain, key)?;
    Ok(warp::reply::json(
        &state.call(AddServerCert { cert }).await?,
    ))
}

/// Delete a certificate.
#[utoipa::path(
    delete,
    path = "/api/server_certs/{id}",
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
    Ok(warp::reply::json(
        &state.call(DeleteServerCert { id }).await?,
    ))
}

/// Download a certificate.
#[utoipa::path(
    get,
    path = "/api/server_certs/{id}/download",
    params(
        ("id" = String, Path, description = "Certification ID")
    ),
    responses(
        (status = 200, body = Vec<u8>),
        (status = 404),
        (status = 401),
    ),
    security(
        ("authorization"=[])
    )
)]
pub async fn download(state: AppState, id: String) -> Result<impl Reply, Rejection> {
    let file = state.call(DownloadServerCert { id: id.clone() }).await?;
    Ok(Response::builder()
        .header("Content-Type", "application/gzip")
        .header(
            "Content-Disposition",
            &format!("attachment; filename=\"{}.tar.gz\"", id),
        )
        .body(file.deref().clone())
        .unwrap())
}
