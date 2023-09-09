use super::{with_state, AppState};
use crate::server::rpc::acme::*;
use taxy_api::{
    acme::{AcmeConfig, AcmeRequest},
    id::ShortId,
};
use warp::{filters::BoxedFilter, Filter, Rejection, Reply};

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

    let api_add = warp::post().and(
        with_state(app_state.clone())
            .and(warp::body::json())
            .and(warp::path::end())
            .and_then(add),
    );

    let api_put = warp::put().and(
        with_state(app_state.clone())
            .and(warp::body::json())
            .and(warp::path::param())
            .and(warp::path::end())
            .and_then(put),
    );

    let api_delete = warp::delete().and(
        with_state(app_state)
            .and(warp::path::param())
            .and(warp::path::end())
            .and_then(delete),
    );

    warp::path("acme")
        .and(api_delete.or(api_get).or(api_add).or(api_put).or(api_list))
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
        ("cookie"=[])
    )
)]
pub async fn list(state: AppState) -> Result<impl Reply, Rejection> {
    Ok(warp::reply::json(&state.call(GetAcmeList).await?))
}

/// Get an ACME configuration.
#[utoipa::path(
    get,
    path = "/api/acme/{id}",
    params(
        ("id" = String, Path, description = "ACME ID")
    ),
    responses(
        (status = 200, body = [AcmeInfo]),
        (status = 404),
        (status = 401),
    ),
    security(
        ("cookie"=[])
    )
)]
pub async fn get(state: AppState, id: ShortId) -> Result<impl Reply, Rejection> {
    Ok(warp::reply::json(&state.call(GetAcme { id }).await?))
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
        ("cookie"=[])
    )
)]
pub async fn add(state: AppState, request: AcmeRequest) -> Result<impl Reply, Rejection> {
    Ok(warp::reply::json(&state.call(AddAcme { request }).await?))
}

/// Update an ACME configuration.
#[utoipa::path(
    put,
    path = "/api/acme/{id}",
    params(
        ("id" = String, Path, description = "ACME ID")
    ),
    request_body = AcmeConfig,
    responses(
        (status = 200),
        (status = 404),
        (status = 400, body = Error),
        (status = 401),
    ),
    security(
        ("cookie"=[])
    )
)]
pub async fn put(
    state: AppState,
    config: AcmeConfig,
    id: ShortId,
) -> Result<impl Reply, Rejection> {
    Ok(warp::reply::json(
        &state.call(UpdateAcme { id, config }).await?,
    ))
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
        ("cookie"=[])
    )
)]
pub async fn delete(state: AppState, id: ShortId) -> Result<impl Reply, Rejection> {
    Ok(warp::reply::json(&state.call(DeleteAcme { id }).await?))
}
