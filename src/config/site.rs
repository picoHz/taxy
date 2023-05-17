use super::subject_name::SubjectName;
use serde_derive::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Site {
    pub id: String,
    #[schema(example = "c56yqmqcvpmp49n14s2lexxl")]
    pub port: String,
    #[schema(value_type = [String], example = json!(["example.com"]))]
    pub vhosts: Vec<SubjectName>,
}
