use serde_default::DefaultFromSerde;
use serde_derive::{Deserialize, Serialize};
use std::{net::SocketAddr, path::PathBuf, time::Duration};
use utoipa::ToSchema;

#[derive(Debug, DefaultFromSerde, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct AppConfig {
    #[serde(with = "humantime_serde", default = "default_background_task_interval")]
    #[schema(value_type = String, example = "1h")]
    pub background_task_interval: Duration,

    #[serde(default)]
    pub admin: AdminConfig,

    #[serde(default = "default_http_challenge_addr")]
    #[schema(value_type = String, example = "0.0.0.0:80")]
    pub http_challenge_addr: SocketAddr,
}

fn default_background_task_interval() -> Duration {
    Duration::from_secs(60 * 60)
}

fn default_http_challenge_addr() -> SocketAddr {
    SocketAddr::from(([0, 0, 0, 0], 80))
}

#[derive(Clone, Serialize, ToSchema)]
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
    #[schema(value_type = String, example = "/home/taxy/.config/taxy")]
    pub log_path: PathBuf,
}

#[derive(Debug, DefaultFromSerde, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct AdminConfig {
    #[serde(with = "humantime_serde", default = "default_admin_session_expiry")]
    #[schema(value_type = String, example = "1d")]
    pub session_expiry: Duration,

    #[serde(default = "default_max_attempts")]
    pub max_login_attempts: u32,

    #[serde(with = "humantime_serde", default = "default_login_attempts_reset")]
    #[schema(value_type = String, example = "15m")]
    pub login_attempts_reset: Duration,
}

fn default_admin_session_expiry() -> Duration {
    Duration::from_secs(60 * 60)
}

fn default_max_attempts() -> u32 {
    5
}

fn default_login_attempts_reset() -> Duration {
    Duration::from_secs(60 * 15)
}
