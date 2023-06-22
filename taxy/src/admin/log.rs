use super::{with_state, AppState};
use sqlx::ConnectOptions;
use sqlx::{sqlite::SqliteConnectOptions, Row, SqlitePool};
use std::path::Path;
use std::time::Duration;
use taxy_api::error::Error;
use taxy_api::log::{LogQuery, SystemLogRow};
use time::OffsetDateTime;
use warp::{filters::BoxedFilter, Filter, Rejection, Reply};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
const REQUEST_INTERVAL: Duration = Duration::from_secs(1);
const REQUEST_DEFAULT_LIMIT: u32 = 100;

pub fn api(app_state: AppState) -> BoxedFilter<(impl Reply,)> {
    warp::get()
        .and(warp::path("log"))
        .and(
            with_state(app_state)
                .and(warp::path::param())
                .and(warp::query())
                .and(warp::path::end())
                .and_then(get),
        )
        .boxed()
}

/// Get log.
#[utoipa::path(
    get,
    path = "/api/log/{id}",
    params(
        ("id" = String, Path, description = "Resource ID"),
        LogQuery
    ),
    responses(
        (status = 200, body = Vec<SystemLogRow>),
        (status = 408),
        (status = 404),
        (status = 401),
    ),
    security(
        ("cookie"=[])
    )
)]
pub async fn get(state: AppState, id: String, query: LogQuery) -> Result<impl Reply, Rejection> {
    let log = state.data.lock().await.log.clone();
    let rows = log
        .fetch_system_log(&id, query.since, query.until, query.limit)
        .await?;
    Ok(warp::reply::json(&rows))
}

pub struct LogReader {
    pool: SqlitePool,
}

impl LogReader {
    pub async fn new(path: &Path) -> anyhow::Result<Self> {
        let mut opt = SqliteConnectOptions::new().filename(path).read_only(true);
        opt.log_statements(log::LevelFilter::Trace);
        let pool = SqlitePool::connect_with(opt).await?;
        Ok(Self { pool })
    }

    pub async fn fetch_system_log(
        &self,
        resource_id: &str,
        since: Option<OffsetDateTime>,
        until: Option<OffsetDateTime>,
        limit: Option<u32>,
    ) -> Result<Vec<SystemLogRow>, Error> {
        let mut timeout = tokio::time::interval(REQUEST_TIMEOUT);
        timeout.tick().await;

        loop {
            let rows = sqlx::query("select * from system_log WHERE resource_id = ? AND (timestamp BETWEEN ? AND ?) ORDER BY timestamp DESC LIMIT ?")
                .bind(resource_id)
                .bind(since.unwrap_or(OffsetDateTime::UNIX_EPOCH))
                .bind(until.unwrap_or_else(OffsetDateTime::now_utc))
                .bind(limit.unwrap_or(REQUEST_DEFAULT_LIMIT))
                .fetch_all(&self.pool);
            tokio::select! {
                _ = timeout.tick() => {
                    break;
                }
                rows = rows => {
                    match rows {
                        Ok(rows) if !rows.is_empty() || until.is_some() => {
                            return Ok(rows
                                .into_iter()
                                .map(|row| SystemLogRow {
                                    timestamp: row.get(0),
                                    level: row.get(1),
                                    resource_id: row.get(2),
                                    message: row.get(3),
                                    fields: serde_json::from_str(row.get(4)).unwrap_or_default(),
                                })
                                .rev()
                                .collect());
                        },
                        Err(_) => {
                            return Err(Error::FailedToFetchLog);
                        }
                        _ => {
                            tokio::time::sleep(REQUEST_INTERVAL).await;
                        }
                    }
                }
            }
        }

        Err(Error::WaitingLogTimedOut)
    }
}
