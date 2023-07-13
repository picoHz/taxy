use serde_default::DefaultFromSerde;
use serde_derive::{Deserialize, Serialize};
use std::time::Duration;
use utoipa::ToSchema;

#[derive(Debug, DefaultFromSerde, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct HealthCheck {
    #[serde(with = "humantime_serde", default = "default_interval")]
    #[schema(value_type = String, example = "10m")]
    pub interval: Duration,

    #[serde(with = "humantime_serde", default = "default_timeout")]
    #[schema(value_type = String, example = "30s")]
    pub timeout: Duration,

    #[serde(default = "default_passes_fails")]
    pub passes: u32,

    #[serde(default = "default_passes_fails")]
    pub fails: u32,

    #[serde(flatten, default = "default_protocol")]
    #[schema(inline)]
    pub protocol: HealthCheckProtocol,
}

fn default_interval() -> Duration {
    Duration::from_secs(60 * 10)
}

fn default_timeout() -> Duration {
    Duration::from_secs(30)
}

fn default_passes_fails() -> u32 {
    1
}

fn default_protocol() -> HealthCheckProtocol {
    HealthCheckProtocol::Tcp(TcpHealthCheck::default())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case", tag = "protocol")]
pub enum HealthCheckProtocol {
    Tcp(TcpHealthCheck),
}

#[derive(Debug, DefaultFromSerde, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct TcpHealthCheck {
    #[serde(default)]
    pub send: String,

    #[serde(default)]
    pub expect: String,
}
