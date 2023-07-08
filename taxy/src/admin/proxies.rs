use super::{with_state, AppState};
use crate::server::rpc::proxies::*;
use taxy_api::site::Proxy;
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

    let api_delete = warp::delete().and(
        with_state(app_state.clone())
            .and(warp::path::param())
            .and(warp::path::end())
            .and_then(delete),
    );

    let api_put = warp::put().and(
        with_state(app_state.clone())
            .and(warp::body::json())
            .and(warp::path::param())
            .and(warp::path::end())
            .and_then(put),
    );

    let api_post = warp::post().and(
        with_state(app_state)
            .and(warp::body::json())
            .and(warp::path::end())
            .and_then(post),
    );

    warp::path("proxies")
        .and(api_delete.or(api_get).or(api_put).or(api_list).or(api_post))
        .boxed()
}

/// Get the list of site configurations.
#[utoipa::path(
    get,
    path = "/api/proxies",
    responses(
        (status = 200, body = [ProxyEntry])
    ),
    responses(
        (status = 401),
    ),
    security(
        ("cookie"=[])
    )
)]
pub async fn list(state: AppState) -> Result<impl Reply, Rejection> {
    Ok(warp::reply::json(&state.call(GetProxyList).await?))
}

/// Get a site configuration.
#[utoipa::path(
    get,
    path = "/api/proxies/{id}",
    params(
        ("id" = String, Path, description = "Port configuration id")
    ),
    responses(
        (status = 200, body = ProxyEntry),
        (status = 404),
        (status = 401),
    ),
    security(
        ("cookie"=[])
    )
)]
pub async fn get(state: AppState, id: String) -> Result<impl Reply, Rejection> {
    Ok(warp::reply::json(&state.call(GetProxy { id }).await?))
}

/// Delete a site configuration.
#[utoipa::path(
    delete,
    path = "/api/proxies/{id}",
    params(
        ("id" = String, Path, description = "Proxy configuration id")
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
pub async fn delete(state: AppState, id: String) -> Result<impl Reply, Rejection> {
    Ok(warp::reply::json(&state.call(DeleteProxy { id }).await?))
}

/// Create a new site configuration.
#[utoipa::path(
    post,
    path = "/api/proxies",
    request_body = ProxyEntry,
    responses(
        (status = 200),
        (status = 400, body = Error),
        (status = 401),
    ),
    security(
        ("cookie"=[])
    )
)]
pub async fn post(state: AppState, entry: Proxy) -> Result<impl Reply, Rejection> {
    Ok(warp::reply::json(&state.call(AddProxy { entry }).await?))
}

/// Update a site configuration.
#[utoipa::path(
    put,
    path = "/api/proxies/{id}",
    params(
        ("id" = String, Path, description = "Proxy configuration name")
    ),
    request_body = ProxyEntry,
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
pub async fn put(state: AppState, entry: Proxy, id: String) -> Result<impl Reply, Rejection> {
    let entry = (id, entry).into();
    Ok(warp::reply::json(&state.call(UpdateProxy { entry }).await?))
}
