use clap::ValueEnum;
use sqlx::Executor;
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use std::io::{self, Cursor};
use std::path::{Path, PathBuf};
use tracing::{
    field::{Field, Visit},
    Event,
};
use tracing::{span, Subscriber};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::fmt;
use tracing_subscriber::fmt::format::JsonVisitor;
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

use tracing_subscriber::fmt::format::DefaultFields;

pub struct DynamicFileLayer {}

impl DynamicFileLayer {
    pub async fn new(path: &Path) -> Self {
        let opt = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true);
        let mut conn = SqlitePool::connect_with(opt)
            .await
            .unwrap()
            .acquire()
            .await
            .unwrap();

        {
            let opt = SqliteConnectOptions::new()
                .filename(path)
                .create_if_missing(true);
            let pool = SqlitePool::connect_with(opt).await.unwrap();
            let mut conn = pool.acquire().await.unwrap();

            tokio::spawn(async move {
                conn.execute(
                    sqlx::query("INSERT INTO todos (description) VALUES($1);").bind(cuid2::cuid()),
                )
                .await
                .unwrap();
            });
        }

        conn.execute(sqlx::query(
            "CREATE TABLE IF NOT EXISTS todos
        (
            id          INTEGER PRIMARY KEY NOT NULL,
            description TEXT                NOT NULL,
            done        BOOLEAN             NOT NULL DEFAULT 0
        );",
        ))
        .await
        .unwrap();

        DynamicFileLayer {}
    }
}

impl<S> Layer<S> for DynamicFileLayer
where
    S: Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    fn on_new_span(&self, attrs: &span::Attributes<'_>, id: &span::Id, ctx: Context<'_, S>) {
        let mut visitor = KeyValueVisitor::new("remote");
        attrs.record(&mut visitor);
    }

    fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
        let metadata = event.metadata();

        let mut buf = String::new();
        {
            let mut visitor = JsonVisitor::new(&mut buf);
            event.record(&mut visitor);
        }

        println!("buf: {:?}", buf);

        let mut visitor = KeyValueVisitor::new("remote");
        event.record(&mut visitor);

        if let Some(span) = ctx.lookup_current() {
            let ext = span.extensions();
            let fields = ext.get::<DefaultFields>();
            println!("fields: {:?} {:?}", metadata.target(), visitor.get_value());
        }
    }
}

struct KeyValueVisitor {
    key: &'static str,
    value: Option<String>,
}

impl KeyValueVisitor {
    fn new(key: &'static str) -> Self {
        KeyValueVisitor { key, value: None }
    }

    fn get_value(self) -> Option<String> {
        self.value
    }
}

impl Visit for KeyValueVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        println!("record_debug: {:?} {:?}", field.name(), value);
        if field.name() == self.key {
            self.value = Some(format!("{:?}", value));
        }
    }
}
