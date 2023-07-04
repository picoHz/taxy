use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use time::OffsetDateTime;
use utoipa::{IntoParams, ToSchema};

#[derive(Clone, Copy, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl TryFrom<u8> for LogLevel {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, ()> {
        match value {
            1 => Ok(LogLevel::Error),
            2 => Ok(LogLevel::Warn),
            3 => Ok(LogLevel::Info),
            4 => Ok(LogLevel::Debug),
            5 => Ok(LogLevel::Trace),
            _ => Err(()),
        }
    }
}

impl ToString for LogLevel {
    fn to_string(&self) -> String {
        match self {
            LogLevel::Error => "error".to_string(),
            LogLevel::Warn => "warn".to_string(),
            LogLevel::Info => "info".to_string(),
            LogLevel::Debug => "debug".to_string(),
            LogLevel::Trace => "trace".to_string(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, ToSchema)]
pub struct SystemLogRow {
    #[serde(
        serialize_with = "serialize_timestamp",
        deserialize_with = "deserialize_timestamp"
    )]
    #[schema(value_type = u64)]
    pub timestamp: OffsetDateTime,
    #[schema(value_type = String, example = "info")]
    pub level: LogLevel,
    pub resource_id: String,
    pub message: String,
    pub fields: HashMap<String, String>,
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
    let timestamp = i64::deserialize(deserializer)?;
    OffsetDateTime::from_unix_timestamp(timestamp).map_err(serde::de::Error::custom)
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct LogQuery {
    #[serde(default, deserialize_with = "deserialize_time")]
    #[param(value_type = Option<u64>)]
    pub since: Option<OffsetDateTime>,
    #[serde(default, deserialize_with = "deserialize_time")]
    #[param(value_type = Option<u64>)]
    pub until: Option<OffsetDateTime>,
    pub limit: Option<u32>,
}

fn deserialize_time<'de, D>(deserializer: D) -> Result<Option<OffsetDateTime>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let timestamp = Option::<i64>::deserialize(deserializer)?;
    Ok(timestamp.and_then(|timestamp| OffsetDateTime::from_unix_timestamp(timestamp).ok()))
}
