use super::{with_state, AppState};
use crate::{
    keyring::acme::{AcmeEntry, AcmeRequest},
    server::rpc::acme::*,
};
use warp::{filters::BoxedFilter, Filter, Rejection, Reply};

pub fn api(app_state: AppState) -> BoxedFilter<(impl Reply,)> {
    let acme_list = warp::get()
        .and(warp::path::end())
        .and(with_state(app_state.clone()).and_then(list));

    let acme_add = warp::post().and(
        with_state(app_state.clone())
            .and(warp::body::json())
            .and(warp::path::end())
            .and_then(add),
    );

    let acme_delete = warp::delete().and(
        with_state(app_state)
            .and(warp::path::param())
            .and(warp::path::end())
            .and_then(delete),
    );

    warp::path("acme")
        .and(acme_delete.or(acme_add).or(acme_list))
        .boxed()
}

/// List ACME configurations.
#[utoipa::path(
    get,
    path = "/api/acme",
    responses(
        (status = 200, body = [AcmeInfo]),
        (status = 401),
    ),
    security(
        ("authorization"=[])
    )
)]
pub async fn list(state: AppState) -> Result<impl Reply, Rejection> {
    Ok(warp::reply::json(&state.call(GetAcmeList).await?))
}

/// Register an ACME configuration.
#[utoipa::path(
    post,
    path = "/api/acme",
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
pub async fn add(state: AppState, request: AcmeRequest) -> Result<impl Reply, Rejection> {
    let item = AcmeEntry::new(request).await?;
    Ok(warp::reply::json(&state.call(AddAcme { item }).await?))
}

/// Delete an ACME configuration.
#[utoipa::path(
    delete,
    path = "/api/acme/{id}",
    params(
        ("id" = String, Path, description = "ACME ID")
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
    Ok(warp::reply::json(&state.call(DeleteAcme { id }).await?))
}
