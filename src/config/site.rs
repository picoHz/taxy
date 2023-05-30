use super::subject_name::SubjectName;
use serde_derive::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Site {
    #[schema(example = json!(["c56yqmqcvpmp49n14s2lexxl"]))]
    pub ports: Vec<String>,
    #[schema(value_type = [String], example = json!(["example.com"]))]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub vhosts: Vec<SubjectName>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct SiteEntry {
    pub id: String,
    #[schema(inline)]
    #[serde(flatten)]
    pub site: Site,
}

impl From<(String, Site)> for SiteEntry {
    fn from((id, site): (String, Site)) -> Self {
        Self { id, site }
    }
}

impl From<SiteEntry> for (String, Site) {
    fn from(entry: SiteEntry) -> Self {
        (entry.id, entry.site)
    }
}
