use super::AppState;
use crate::{config::site::Site, server::rpc::sites::*};
use warp::{Rejection, Reply};

/// Get the list of port configurations.
#[utoipa::path(
    get,
    path = "/api/sites",
    responses(
        (status = 200, body = [SiteEntry])
    ),
    responses(
        (status = 401),
    ),
    security(
        ("authorization"=[])
    )
)]
pub async fn list(state: AppState) -> Result<impl Reply, Rejection> {
    Ok(warp::reply::json(&state.call(GetSiteList).await?))
}

/// Delete a port configuration.
#[utoipa::path(
    delete,
    path = "/api/sites/{id}",
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
    Ok(warp::reply::json(&state.call(DeleteSite { id }).await?))
}

/// Create a new port configuration.
#[utoipa::path(
    post,
    path = "/api/sites",
    request_body = SiteEntry,
    responses(
        (status = 200),
        (status = 400, body = Error),
        (status = 401),
    ),
    security(
        ("authorization"=[])
    )
)]
pub async fn post(state: AppState, entry: Site) -> Result<impl Reply, Rejection> {
    Ok(warp::reply::json(
        &state
            .call(AddSite {
                entry: (cuid2::cuid(), entry).into(),
            })
            .await?,
    ))
}

/// Update or rename a port configuration.
#[utoipa::path(
    put,
    path = "/api/sites/{id}",
    params(
        ("id" = String, Path, description = "Port configuration name")
    ),
    request_body = SiteEntry,
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
pub async fn put(state: AppState, entry: Site, id: String) -> Result<impl Reply, Rejection> {
    let entry = (id, entry).into();
    Ok(warp::reply::json(&state.call(UpdateSite { entry }).await?))
}
