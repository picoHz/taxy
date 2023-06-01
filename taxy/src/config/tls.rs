use serde_derive::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct TlsTermination {
    #[schema(example = json!(["*.example.com"]))]
    pub server_names: Vec<String>,
}
