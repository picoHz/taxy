use serde::Serialize;
use sqlx::{sqlite::SqliteConnectOptions, Row, SqlitePool};
use std::{collections::HashMap, path::Path};
use time::OffsetDateTime;
use utoipa::ToSchema;

pub struct LogReader {
    pool: SqlitePool,
}

impl LogReader {
    pub async fn new(path: &Path) -> anyhow::Result<Self> {
        let opt = SqliteConnectOptions::new().filename(path).read_only(true);
        let pool = SqlitePool::connect_with(opt).await?;
        Ok(Self { pool })
    }

    pub async fn fetch_system_log(&self, resource_id: &str) -> anyhow::Result<Vec<SystemLogRow>> {
        let rows = sqlx::query("select * from system_log WHERE resource_id = ? AND (timestamp BETWEEN ? AND ?) ORDER BY timestamp")
            .bind(resource_id)
            .bind(OffsetDateTime::UNIX_EPOCH)
            .bind(OffsetDateTime::now_utc())
            .fetch_all(&self.pool).await?;
        Ok(rows
            .into_iter()
            .map(|row| SystemLogRow {
                timestamp: row.get(0),
                level: row.get(1),
                resource_id: row.get(2),
                message: row.get(3),
                fields: serde_json::from_str(row.get(4)).unwrap_or_default(),
            })
            .collect())
    }
}

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
