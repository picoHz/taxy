use serde_derive::{Deserialize, Serialize};
use utoipa::ToSchema;

pub mod port;
pub mod storage;
pub mod tls;

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct AppConfig {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    File,
    Api,
}
