use serde_default::DefaultFromSerde;
use serde_derive::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum TlsState {
    Active,
}

#[derive(Debug, Clone, DefaultFromSerde, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct TlsTermination {
    #[serde(default)]
    #[schema(example = json!(["*.example.com"]))]
    pub server_names: Vec<String>,
}
