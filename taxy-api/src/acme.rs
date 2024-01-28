use crate::{id::ShortId, subject_name::SubjectName};
use base64::{engine::general_purpose, Engine as _};
use serde_default::DefaultFromSerde;
use serde_derive::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, ToSchema)]
pub struct Acme {
    #[schema(inline)]
    #[serde(flatten)]
    pub config: AcmeConfig,
    #[schema(value_type = [String], example = json!(["example.com"]))]
    pub identifiers: Vec<SubjectName>,
    #[schema(value_type = String, example = "http-01")]
    pub challenge_type: String,
}

#[derive(Debug, DefaultFromSerde, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct AcmeConfig {
    #[serde(default = "default_active", skip_serializing_if = "is_true")]
    pub active: bool,
    #[serde(default)]
    #[schema(example = "Let's Encrypt")]
    pub provider: String,
    #[schema(example = "60")]
    #[serde(default = "default_renewal_days")]
    pub renewal_days: u64,
}

fn default_active() -> bool {
    true
}

fn is_true(b: &bool) -> bool {
    *b
}

fn default_renewal_days() -> u64 {
    60
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct AcmeInfo {
    pub id: ShortId,
    #[schema(inline)]
    #[serde(flatten)]
    pub config: AcmeConfig,
    #[schema(example = json!(["example.com"]))]
    pub identifiers: Vec<String>,
    #[schema(value_type = String, example = "http-01")]
    pub challenge_type: String,
    pub next_renewal: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, ToSchema)]
pub struct AcmeRequest {
    #[schema(example = "https://acme-staging-v02.api.letsencrypt.org/directory")]
    pub server_url: String,
    #[schema(example = json!(["mailto:admin@example.com"]))]
    pub contacts: Vec<String>,
    #[serde(default)]
    pub eab: Option<ExternalAccountBinding>,
    #[schema(inline)]
    #[serde(flatten)]
    pub acme: Acme,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, ToSchema)]
pub struct ExternalAccountBinding {
    #[schema(example = "f9cf7e3faa1aca7e6086")]
    pub key_id: String,
    #[schema(value_type = String, example = "TszzWRgQWTUqo04dxmSuKDH06")]
    #[serde(
        serialize_with = "serialize_hmac_key",
        deserialize_with = "deserialize_hmac_key"
    )]
    pub hmac_key: Vec<u8>,
}

fn serialize_hmac_key<S>(hmac_key: &[u8], serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&general_purpose::URL_SAFE_NO_PAD.encode(hmac_key))
}

fn deserialize_hmac_key<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Deserialize;
    let hmac_key = String::deserialize(deserializer)?;
    general_purpose::URL_SAFE_NO_PAD
        .decode(hmac_key.as_bytes())
        .map_err(serde::de::Error::custom)
}
