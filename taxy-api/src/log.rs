use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use time::OffsetDateTime;
use utoipa::{IntoParams, ToSchema};

#[derive(Serialize, ToSchema)]
pub struct SystemLogRow {
    #[serde(serialize_with = "serialize_timestamp")]
    #[schema(value_type = u64)]
    pub timestamp: OffsetDateTime,
    #[serde(serialize_with = "serialize_level")]
    #[schema(value_type = String, example = "info")]
    pub level: u8,
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

fn serialize_level<S>(level: &u8, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let level = match *level {
        1 => "error",
        2 => "warn",
        3 => "info",
        4 => "debug",
        5 => "trace",
        _ => "unknown",
    };
    serializer.serialize_str(level)
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
