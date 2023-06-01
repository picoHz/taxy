use super::{with_state, AppState};
use crate::server::rpc::sites::*;
use taxy_api::site::Site;
use warp::{filters::BoxedFilter, Filter, Rejection, Reply};

pub fn api(app_state: AppState) -> BoxedFilter<(impl Reply,)> {
    let api_list = warp::get()
        .and(warp::path::end())
        .and(with_state(app_state.clone()).and_then(list));

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

    warp::path("sites")
        .and(api_delete.or(api_put).or(api_list).or(api_post))
        .boxed()
}

/// Get the list of site configurations.
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

/// Delete a site configuration.
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

/// Create a new site configuration.
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

/// Update a site configuration.
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
