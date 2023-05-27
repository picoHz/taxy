use super::{with_state, AppState};
use warp::{filters::BoxedFilter, Filter, Rejection, Reply};

pub fn api(app_state: AppState) -> BoxedFilter<(impl Reply,)> {
    warp::path("app_info")
        .and(warp::get())
        .and(with_state(app_state).and(warp::path::end()).and_then(get))
        .boxed()
}

/// Get app info.
#[utoipa::path(
    get,
    path = "/api/app_info",
    responses(
        (status = 200, body = AppInfo),
        (status = 401),
    ),
    security(
        ("authorization"=[])
    )
)]
pub async fn get(state: AppState) -> Result<impl Reply, Rejection> {
    Ok(warp::reply::json(&state.data.lock().await.app_info))
}
