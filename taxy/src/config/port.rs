use super::tls::TlsTermination;
use multiaddr::Multiaddr;
use serde_derive::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct UpstreamServer {
    #[schema(value_type = String, example = "/dns/example.com/tcp/8080")]
    pub addr: Multiaddr,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct PortEntry {
    pub id: String,
    #[schema(inline)]
    #[serde(flatten)]
    pub port: Port,
}

impl From<(String, Port)> for PortEntry {
    fn from((id, port): (String, Port)) -> Self {
        Self { id, port }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Port {
    #[schema(value_type = String, example = "/ip4/127.0.0.1/tcp/8080")]
    pub listen: Multiaddr,
    #[serde(flatten, default)]
    pub opts: PortOptions,
}

impl From<PortEntry> for (String, Port) {
    fn from(entry: PortEntry) -> Self {
        (entry.id, entry.port)
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct PortOptions {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub upstream_servers: Vec<UpstreamServer>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tls_termination: Option<TlsTermination>,
}
