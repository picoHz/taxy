use clap::ValueEnum;
use std::path::PathBuf;
use tracing::Subscriber;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::fmt;
use tracing_subscriber::fmt::writer::MakeWriterExt;
use tracing_subscriber::layer;
use tracing_subscriber::layer::Layer;
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
