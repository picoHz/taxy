use super::subject_name::SubjectName;
use serde_derive::{Deserialize, Serialize};
use url::Url;
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Site {
    #[schema(example = json!(["c56yqmqcvpmp49n14s2lexxl"]))]
    pub ports: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[schema(value_type = [String], example = json!(["example.com"]))]
    pub vhosts: Vec<SubjectName>,
    pub routes: Vec<Route>,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Route {
    #[schema(example = "/")]
    #[serde(default = "default_route_path")]
    pub path: String,
    pub servers: Vec<Server>,
}

fn default_route_path() -> String {
    "/".to_owned()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Server {
    #[schema(value_type = String, example = "https://example.com/api")]
    pub url: Url,
}
