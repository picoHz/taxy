use serde_derive::Serialize;
use utoipa::ToSchema;
use warp::{Rejection, Reply};

/// Get app info.
#[utoipa::path(
    get,
    path = "/api/app_info",
    responses(
        (status = 200, body = AppInfo)
    )
)]
pub async fn get() -> Result<impl Reply, Rejection> {
    Ok(warp::reply::json(&AppInfo::new()))
}

#[derive(Serialize, ToSchema)]
pub struct AppInfo {
    #[schema(example = "0.0.0")]
    version: &'static str,
    #[schema(example = "aarch64-apple-darwin")]
    target: &'static str,
    #[schema(example = "debug")]
    profile: &'static str,
    #[schema(example = json!([]))]
    features: &'static [&'static str],
    #[schema(example = "rustc 1.69.0 (84c898d65 2023-04-16)")]
    rustc: &'static str,
}

mod build_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

impl AppInfo {
    fn new() -> Self {
        Self {
            version: build_info::PKG_VERSION,
            target: build_info::TARGET,
            profile: build_info::PROFILE,
            features: &build_info::FEATURES[..],
            rustc: build_info::RUSTC_VERSION,
        }
    }
}
