use crate::error::Error;
use crate::subject_name::SubjectName;
use crate::{id::ShortId, port::UpstreamServer};
use serde_default::DefaultFromSerde;
use serde_derive::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use url::Url;
use utoipa::ToSchema;

#[derive(Debug, DefaultFromSerde, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Proxy {
    #[serde(default = "default_active", skip_serializing_if = "is_true")]
    pub active: bool,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub name: String,
    #[serde(default)]
    #[schema(example = json!(["c56yqmqcvpmp49n14s2lexxl"]))]
    pub ports: Vec<ShortId>,
    #[serde(flatten, default = "default_kind")]
    #[schema(inline)]
    pub kind: ProxyKind,
}

fn default_active() -> bool {
    true
}

fn is_true(b: &bool) -> bool {
    *b
}

fn default_kind() -> ProxyKind {
    ProxyKind::Http(HttpProxy::default())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(tag = "protocol", rename_all = "snake_case")]
pub enum ProxyKind {
    Tcp(TcpProxy),
    Http(HttpProxy),
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct TcpProxy {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub upstream_servers: Vec<UpstreamServer>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct HttpProxy {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[schema(value_type = [String], example = json!(["example.com"]))]
    pub vhosts: Vec<SubjectName>,
    pub routes: Vec<Route>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProxyState {
    Active,
    Inactive,
    #[default]
    Unknown,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ProxyStatus {
    pub state: ProxyState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ProxyEntry {
    pub id: ShortId,
    #[schema(inline)]
    #[serde(flatten)]
    pub proxy: Proxy,
}

impl From<(ShortId, Proxy)> for ProxyEntry {
    fn from((id, proxy): (ShortId, Proxy)) -> Self {
        Self { id, proxy }
    }
}

impl From<ProxyEntry> for (ShortId, Proxy) {
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
    pub url: ServerUrl,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
#[serde(transparent)]
#[schema(value_type = String)]
pub struct ServerUrl(Url);

impl ServerUrl {
    pub fn hostname(&self) -> Option<&str> {
        self.0.host_str()
    }

    pub fn authority(&self) -> Option<String> {
        Some(format!(
            "{}:{}",
            self.hostname()?,
            self.0.port_or_known_default().unwrap_or_default()
        ))
    }
}

impl<'de> serde::Deserialize<'de> for ServerUrl {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let url = String::deserialize(deserializer)?;
        ServerUrl::from_str(&url).map_err(serde::de::Error::custom)
    }
}

impl From<ServerUrl> for Url {
    fn from(url: ServerUrl) -> Self {
        url.0
    }
}

impl FromStr for ServerUrl {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Url::from_str(s)
            .ok()
            .map(ServerUrl)
            .filter(|url| url.authority().is_some())
            .ok_or_else(|| Error::InvalidServerUrl { url: s.into() })
    }
}

impl TryFrom<Url> for ServerUrl {
    type Error = url::ParseError;

    fn try_from(url: Url) -> Result<Self, Self::Error> {
        Ok(ServerUrl(url))
    }
}

impl fmt::Display for ServerUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
