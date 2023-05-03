use warp::{Rejection, Reply};

use super::AppState;

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
