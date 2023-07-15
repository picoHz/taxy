use serde_default::DefaultFromSerde;
use serde_derive::{Deserialize, Serialize};
use std::time::Duration;
use time::OffsetDateTime;
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct HealthCheckResult {
    pub alive: bool,
    #[serde(
        serialize_with = "serialize_timestamp",
        deserialize_with = "deserialize_timestamp"
    )]
    #[schema(value_type = u64)]
    pub timestamp: OffsetDateTime,
}

fn serialize_timestamp<S>(timestamp: &OffsetDateTime, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_i64(timestamp.unix_timestamp())
}

fn deserialize_timestamp<'de, D>(deserializer: D) -> Result<OffsetDateTime, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    let timestamp = i64::deserialize(deserializer)?;
    OffsetDateTime::from_unix_timestamp(timestamp).map_err(serde::de::Error::custom)
}
