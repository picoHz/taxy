use super::{with_state, AppState};
use crate::server::rpc::ports::*;
use taxy_api::port::Port;
use warp::{filters::BoxedFilter, Filter, Rejection, Reply};

pub fn api(app_state: AppState) -> BoxedFilter<(impl Reply,)> {
    let api_list = warp::get()
        .and(warp::path::end())
        .and(with_state(app_state.clone()).and_then(list));

    let api_get = warp::get()
        .and(with_state(app_state.clone()))
        .and(warp::path::param())
        .and(warp::path::end())
        .and_then(get);

    let api_status = warp::get()
        .and(with_state(app_state.clone()))
        .and(warp::path::param())
        .and(warp::path("status"))
        .and(warp::path::end())
        .and_then(status);

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
        with_state(app_state.clone())
            .and(warp::body::json())
            .and(warp::path::end())
            .and_then(post),
    );

    let api_reset = warp::get()
        .and(with_state(app_state.clone()))
        .and(warp::path::param())
        .and(warp::path("reset"))
        .and(warp::path::end())
        .and_then(reset);

    let api_interfaces = warp::get()
        .and(with_state(app_state))
        .and(warp::path("interfaces"))
        .and(warp::path::end())
        .and_then(interfaces);

    warp::path("ports")
        .and(
            api_delete
                .or(api_get)
                .or(api_put)
                .or(api_status)
                .or(api_interfaces)
                .or(api_reset)
                .or(api_list)
                .or(api_post),
        )
        .boxed()
}

/// Get the list of port configurations.
#[utoipa::path(
    get,
    path = "/api/ports",
    responses(
        (status = 200, body = [PortEntry])
    ),
    responses(
        (status = 401),
    ),
    security(
        ("cookie"=[])
    )
)]
pub async fn list(state: AppState) -> Result<impl Reply, Rejection> {
    Ok(warp::reply::json(&state.call(GetPortList).await?))
}

/// Get a port configuration.
#[utoipa::path(
    get,
    path = "/api/ports/{id}/status",
    params(
        ("id" = String, Path, description = "Port configuration id")
    ),
    responses(
        (status = 200, body = PortEntry),
        (status = 404),
        (status = 401),
    ),
    security(
        ("cookie"=[])
    )
)]
pub async fn get(state: AppState, id: String) -> Result<impl Reply, Rejection> {
    Ok(warp::reply::json(&state.call(GetPort { id }).await?))
}

/// Get the status of a port.
#[utoipa::path(
    get,
    path = "/api/ports/{id}/status",
    params(
        ("id" = String, Path, description = "Port configuration id")
    ),
    responses(
        (status = 200, body = PortStatus),
        (status = 404),
        (status = 401),
    ),
    security(
        ("cookie"=[])
    )
)]
pub async fn status(state: AppState, id: String) -> Result<impl Reply, Rejection> {
    Ok(warp::reply::json(&state.call(GetPortStatus { id }).await?))
}

/// Delete a port configuration.
#[utoipa::path(
    delete,
    path = "/api/ports/{id}",
    params(
        ("id" = String, Path, description = "Port configuration id")
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
    Ok(warp::reply::json(&state.call(DeletePort { id }).await?))
}

/// Create a new port configuration.
#[utoipa::path(
    post,
    path = "/api/ports",
    request_body = PortEntry,
    responses(
        (status = 200),
        (status = 400, body = Error),
        (status = 401),
    ),
    security(
        ("cookie"=[])
    )
)]
pub async fn post(state: AppState, entry: Port) -> Result<impl Reply, Rejection> {
    Ok(warp::reply::json(&state.call(AddPort { entry }).await?))
}

/// Update or rename a port configuration.
#[utoipa::path(
    put,
    path = "/api/ports/{id}",
    params(
        ("id" = String, Path, description = "Port configuration name")
    ),
    request_body = PortEntry,
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
pub async fn put(state: AppState, entry: Port, id: String) -> Result<impl Reply, Rejection> {
    let entry = (id, entry).into();
    Ok(warp::reply::json(&state.call(UpdatePort { entry }).await?))
}

/// Close all existing connections.
#[utoipa::path(
    get,
    path = "/api/ports/{id}/reset",
    params(
        ("id" = String, Path, description = "Port configuration id")
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
pub async fn reset(state: AppState, id: String) -> Result<impl Reply, Rejection> {
    Ok(warp::reply::json(&state.call(ResetPort { id }).await?))
}

/// Get the list of network interfaces.
#[utoipa::path(
    get,
    path = "/api/ports/interfaces",
    responses(
        (status = 200, body = [NetworkInterface]),
        (status = 401),
    ),
    security(
        ("cookie"=[])
    )
)]
pub async fn interfaces(state: AppState) -> Result<impl Reply, Rejection> {
    Ok(warp::reply::json(
        &state.call(GetNetworkInterfaceList).await?,
    ))
}
