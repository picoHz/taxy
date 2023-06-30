use crate::subject_name::SubjectName;
use serde_default::DefaultFromSerde;
use serde_derive::{Deserialize, Serialize};
use std::fmt;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CertKind {
    Server,
    Client,
    Root,
}

impl fmt::Display for CertKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                CertKind::Server => "server",
                CertKind::Client => "client",
                CertKind::Root => "root",
            }
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct CertInfo {
    #[schema(example = "a13e1ecc080e42cfcdd5")]
    pub id: String,
    pub kind: CertKind,
    #[schema(example = "a13e1ecc080e42cfcdd5b77fec8450c777554aa7269c029b242a7c548d0d73da")]
    pub fingerprint: String,
    #[schema(example = "CN=taxy self signed cert")]
    pub issuer: String,
    pub root_cert: Option<String>,
    #[schema(value_type = [String], example = json!(["localhost"]))]
    pub san: Vec<SubjectName>,
    #[schema(example = "67090118400")]
    pub not_after: i64,
    #[schema(example = "157766400")]
    pub not_before: i64,
    pub is_ca: bool,
    pub metadata: Option<CertMetadata>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct SelfSignedCertRequest {
    #[schema(value_type = [String], example = json!(["localhost"]))]
    pub san: Vec<SubjectName>,
    #[schema(example = "f9cf7e3faa1aca7e6086")]
    pub ca_cert: Option<String>,
}

#[derive(DefaultFromSerde, Clone, Serialize, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct UploadQuery {
    #[serde(default = "default_cert_kind")]
    pub kind: CertKind,
}

fn default_cert_kind() -> CertKind {
    CertKind::Server
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, ToSchema)]
pub struct CertMetadata {
    pub acme_id: String,
    #[serde(
        serialize_with = "serialize_created_at",
        deserialize_with = "deserialize_created_at"
    )]
    #[schema(value_type = u64)]
    pub created_at: SystemTime,
    #[serde(default)]
    pub is_trusted: bool,
}

fn serialize_created_at<S>(time: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let timestamp = time
        .duration_since(UNIX_EPOCH)
        .map_err(|_| serde::ser::Error::custom("invalid timestamp"))?;
    serializer.serialize_u64(timestamp.as_secs())
}

fn deserialize_created_at<'de, D>(deserializer: D) -> Result<SystemTime, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    let timestamp = u64::deserialize(deserializer)?;
    Ok(UNIX_EPOCH + Duration::from_secs(timestamp))
}

#[derive(ToSchema)]
pub struct CertPostBody {
    #[schema(format = Binary)]
    pub chain: String,
    #[schema(format = Binary)]
    pub key: String,
}
