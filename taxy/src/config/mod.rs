use std::path::Path;

use taxy_api::app::AppInfo;

pub mod storage;

mod build_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

pub fn new_appinfo(config_path: &Path, log_path: &Path) -> AppInfo {
    AppInfo {
        version: build_info::PKG_VERSION,
        target: build_info::TARGET,
        profile: build_info::PROFILE,
        features: &build_info::FEATURES[..],
        rustc: build_info::RUSTC_VERSION,
        config_path: config_path.to_owned(),
        log_path: log_path.to_owned(),
    }
}
