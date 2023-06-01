use super::{with_state, AppState};
use crate::server::rpc::config::*;
use taxy_api::app::AppConfig;
use warp::{filters::BoxedFilter, Filter, Rejection, Reply};

pub fn api(app_state: AppState) -> BoxedFilter<(impl Reply,)> {
    let api_get = warp::get()
        .and(warp::path::end())
        .and(with_state(app_state.clone()).and_then(get));

    let api_put = warp::put()
        .and(warp::path::end())
        .and(with_state(app_state).and(warp::body::json()).and_then(put));
    warp::path("config").and(api_get.or(api_put)).boxed()
}

/// Get the application configuration.
#[utoipa::path(
    get,
    path = "/api/config",
    responses(
        (status = 200, body = AppConfig),
        (status = 401),
    ),
    security(
        ("authorization"=[])
    )
)]
pub async fn get(state: AppState) -> Result<impl Reply, Rejection> {
    Ok(warp::reply::json(&state.call(GetConfig).await?))
}

/// Update the application configuration.
#[utoipa::path(
    put,
    path = "/api/config",
    request_body = AppConfig,
    responses(
        (status = 200),
        (status = 400, body = Error),
        (status = 401),
    ),
    security(
        ("authorization"=[])
    )
)]
pub async fn put(state: AppState, config: AppConfig) -> Result<impl Reply, Rejection> {
    Ok(warp::reply::json(&state.call(SetConfig { config }).await?))
}
