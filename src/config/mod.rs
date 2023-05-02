use serde_default::DefaultFromSerde;
use serde_derive::{Deserialize, Serialize};
use std::time::Duration;
use utoipa::ToSchema;

pub mod port;
pub mod storage;
pub mod tls;

#[derive(Debug, DefaultFromSerde, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct AppConfig {
    #[serde(with = "humantime_serde", default = "default_background_task_interval")]
    #[schema(value_type = String, example = "1h")]
    pub background_task_interval: Duration,
}

fn default_background_task_interval() -> Duration {
    Duration::from_secs(60 * 60)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    File,
    Api,
}
