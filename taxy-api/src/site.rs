use crate::port::UpstreamServer;
use crate::subject_name::SubjectName;
use serde_derive::{Deserialize, Serialize};
use url::Url;
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Proxy {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub name: String,
    #[schema(example = json!(["c56yqmqcvpmp49n14s2lexxl"]))]
    pub ports: Vec<String>,
    #[serde(flatten)]
    #[schema(inline)]
    pub kind: ProxyKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(tag = "protocol", rename_all = "snake_case")]
pub enum ProxyKind {
    Tcp(TcpProxy),
    Http(HttpProxy),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct TcpProxy {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub upstream_servers: Vec<UpstreamServer>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct HttpProxy {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[schema(value_type = [String], example = json!(["example.com"]))]
    pub vhosts: Vec<SubjectName>,
    pub routes: Vec<Route>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ProxyEntry {
    pub id: String,
    #[schema(inline)]
    #[serde(flatten)]
    pub proxy: Proxy,
}

impl From<(String, Proxy)> for ProxyEntry {
    fn from((id, proxy): (String, Proxy)) -> Self {
        Self { id, proxy }
    }
}

impl From<ProxyEntry> for (String, Proxy) {
    fn from(entry: ProxyEntry) -> Self {
        (entry.id, entry.proxy)
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
