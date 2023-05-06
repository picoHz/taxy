use clap::ValueEnum;
use dashmap::DashMap;
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use time::OffsetDateTime;
use tokio::runtime::Handle;
use tracing::{
    field::{Field, Visit},
    Event,
};
use tracing::{span, Subscriber};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::fmt;
use tracing_subscriber::fmt::writer::MakeWriterExt;
use tracing_subscriber::layer;
use tracing_subscriber::layer::{Context, Layer};
use tracing_subscriber::registry::LookupSpan;

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
#[clap(rename_all = "snake_case")]
pub enum LogFormat {
    Text,
    Json,
}

pub fn create_layer<S>(
    file: Option<PathBuf>,
    default_file: &str,
    log_level: LevelFilter,
    format: LogFormat,
) -> (
    Box<dyn Layer<S> + Send + Sync + 'static>,
    Option<WorkerGuard>,
)
where
    S: Subscriber + for<'lookup> LookupSpan<'lookup>,
{
    if let Some(level) = log_level.into_level() {
        if let Some(file) = file {
            let (dir, prefix) = match (
                file.parent(),
                file.file_name().and_then(|name| name.to_str()),
            ) {
                (Some(dir), Some(prefix)) => (dir, prefix),
                _ => (file.as_path(), default_file),
            };
            let file_appender = tracing_appender::rolling::hourly(dir, prefix);
            let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
            (
                if format == LogFormat::Json {
                    fmt::layer()
                        .json()
                        .with_writer(non_blocking.with_max_level(level))
                        .boxed()
                } else {
                    fmt::layer()
                        .with_ansi(false)
                        .with_writer(non_blocking.with_max_level(level))
                        .boxed()
                },
                Some(guard),
            )
        } else {
            (
                if format == LogFormat::Json {
                    fmt::layer()
                        .with_writer(std::io::stdout.with_max_level(level))
                        .json()
                        .boxed()
                } else {
                    fmt::layer()
                        .with_writer(std::io::stdout.with_max_level(level))
                        .boxed()
                },
                None,
            )
        }
    } else {
        (layer::Identity::new().boxed(), None)
    }
}

pub struct DatabaseLayer {
    pool: SqlitePool,
    handle: Handle,
    span_map: DashMap<span::Id, String>,
}

impl DatabaseLayer {
    pub async fn new(path: &Path) -> anyhow::Result<Self> {
        let opt = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true);
        let pool = SqlitePool::connect_with(opt).await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS system_log
        (
            timestamp   INTEGER NOT NULL,
            level       TINYINT NOT NULL,
            resource_id STRING          ,
            message     STRING          ,
            fields      STRING              
        );",
        )
        .execute(&pool)
        .await?;

        Ok(DatabaseLayer {
            pool,
            handle: Handle::current(),
            span_map: DashMap::new(),
        })
    }
}

impl<S> Layer<S> for DatabaseLayer
where
    S: Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    fn on_new_span(&self, attrs: &span::Attributes<'_>, id: &span::Id, _ctx: Context<'_, S>) {
        let mut visitor = KeyValueVisitor::default();
        attrs.record(&mut visitor);
        if let Some(resource_id) = visitor.values.remove("resource_id") {
            self.span_map.insert(id.clone(), resource_id);
        }
    }

    fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
        let metadata = event.metadata();
        if metadata.target().starts_with("taxy::access_log") {
            return;
        }

        if let Some(span) = ctx.lookup_current() {
            if let Some(entry) = self.span_map.get(&span.id()) {
                let resource_id = entry.value().to_string();
                let timestamp = OffsetDateTime::now_utc();
                let level = match *metadata.level() {
                    tracing::Level::ERROR => 1,
                    tracing::Level::WARN => 2,
                    tracing::Level::INFO => 3,
                    tracing::Level::DEBUG => 4,
                    tracing::Level::TRACE => 5,
                };
                let mut visitor = KeyValueVisitor::default();
                event.record(&mut visitor);
                let message = visitor.values.remove("message").unwrap_or_default();

                let pool = self.pool.clone();
                self.handle.spawn(async move {
                    sqlx::query(
                        "INSERT INTO system_log (timestamp, level, resource_id, message, fields)
                    VALUES (?, ?, ?, ?, ?)",
                    )
                    .bind(timestamp)
                    .bind(level)
                    .bind(resource_id)
                    .bind(message)
                    .bind(serde_json::to_string(&visitor.values).unwrap_or_default())
                    .execute(&pool)
                    .await
                });
            }
        }
    }
}

#[derive(Default)]
struct KeyValueVisitor {
    pub values: HashMap<String, String>,
}

impl Visit for KeyValueVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        self.values
            .insert(field.name().to_string(), format!("{:?}", value));
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.values
            .insert(field.name().to_string(), value.to_string());
    }
}
