use serde_derive::Serialize;
use warp::{Rejection, Reply};

pub async fn get() -> Result<impl Reply, Rejection> {
    Ok(warp::reply::json(&AppInfo::new()))
}

#[derive(Serialize)]
struct AppInfo {
    version: &'static str,
    target: &'static str,
    profile: &'static str,
    features: &'static [&'static str],
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
