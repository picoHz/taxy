use serde_default::DefaultFromSerde;
use serde_derive::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    time::Duration,
};
use utoipa::ToSchema;

pub mod port;
pub mod storage;
pub mod tls;

#[derive(Debug, DefaultFromSerde, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct AppConfig {
    #[serde(with = "humantime_serde", default = "default_background_task_interval")]
    #[schema(value_type = String, example = "1h")]
    pub background_task_interval: Duration,

    #[serde(with = "humantime_serde", default = "default_admin_session_expiry")]
    #[schema(value_type = String, example = "1d")]
    pub admin_session_expiry: Duration,
}

fn default_background_task_interval() -> Duration {
    Duration::from_secs(60 * 60)
}

fn default_admin_session_expiry() -> Duration {
    Duration::from_secs(60 * 60)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    File,
    Api,
}

#[derive(Serialize, ToSchema)]
pub struct AppInfo {
    #[schema(example = "0.0.0")]
    pub version: &'static str,
    #[schema(example = "aarch64-apple-darwin")]
    pub target: &'static str,
    #[schema(example = "debug")]
    pub profile: &'static str,
    #[schema(example = json!([]))]
    pub features: &'static [&'static str],
    #[schema(example = "rustc 1.69.0 (84c898d65 2023-04-16)")]
    pub rustc: &'static str,
    #[schema(value_type = String, example = "/home/taxy/.config/taxy")]
    pub config_path: PathBuf,
}

mod build_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

impl AppInfo {
    pub fn new(config_path: &Path) -> Self {
        Self {
            version: build_info::PKG_VERSION,
            target: build_info::TARGET,
            profile: build_info::PROFILE,
            features: &build_info::FEATURES[..],
            rustc: build_info::RUSTC_VERSION,
            config_path: config_path.to_owned(),
        }
    }
}
