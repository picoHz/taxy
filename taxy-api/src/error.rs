use multiaddr::Multiaddr;
use serde_derive::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ErrorMessage {
    pub message: String,
    pub error: Option<Error>,
}

#[derive(Debug, Clone, Error, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case", tag = "message")]
pub enum Error {
    #[error("invalid listening address: {addr}")]
    InvalidListeningAddress {
        #[schema(value_type = [String])]
        addr: Multiaddr,
    },

    #[error("invalid server address: {addr}")]
    InvalidServerAddress {
        #[schema(value_type = [String])]
        addr: Multiaddr,
    },

    #[error("invalid subject name: {name}")]
    InvalidSubjectName { name: String },

    #[error("missing TLS termination config")]
    TlsTerminationConfigMissing,

    #[error("failed to generate self-signed certificate")]
    FailedToGerateSelfSignedCertificate,

    #[error("failed to read certificate")]
    FailedToReadCertificate,

    #[error("failed to read private key")]
    FailedToReadPrivateKey,

    #[error("certificate already exists: {id}")]
    CertAlreadyExists { id: String },

    #[error("port id not found: {id}")]
    IdNotFound { id: String },

    #[error("port id already exists: {id}")]
    IdAlreadyExists { id: String },

    #[error("acme account creation failed")]
    AcmeAccountCreationFailed,

    #[error("unauthorized")]
    Unauthorized,

    #[error("failed to create account")]
    FailedToCreateAccount,

    #[error("invalid login credentials")]
    InvalidLoginCredentials,

    #[error("failed to fetch log")]
    FailedToFetchLog,

    #[error("waiting log timed out")]
    WaitingLogTimedOut,

    #[error("rpc error")]
    RpcError,

    #[error("failed to load app key")]
    FailedToLoadAppKey,

    #[error("failed to encrypt private key")]
    FailedToEncryptPrivateKey,

    #[error("failed to decrypt private key")]
    FailedToDecryptPrivateKey,
}

#[cfg(feature = "warp")]
impl warp::reject::Reject for Error {}

impl Error {
    pub fn status_code(&self) -> u16 {
        match self {
            Self::IdNotFound { .. } => 404,
            Self::Unauthorized => 401,
            Self::WaitingLogTimedOut => 408,
            _ => 400,
        }
    }
}
