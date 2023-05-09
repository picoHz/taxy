use hyper::StatusCode;
use multiaddr::Multiaddr;
use serde_derive::Serialize;
use thiserror::Error;
use utoipa::ToSchema;
use warp::reject::Reject;

#[derive(Debug, Clone, Error, Serialize, ToSchema)]
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

    #[error("certificate not found: {id}")]
    KeyringItemNotFound { id: String },

    #[error("no backend servers")]
    EmptyBackendServers,

    #[error("port id not found: {id}")]
    IdNotFound { id: String },

    #[error("acme account creation failed")]
    AcmeAccountCreationFailed,

    #[error("unauthorized")]
    Unauthorized,

    #[error("invalid login credentials")]
    InvalidLoginCredentials,

    #[error("failed to fetch log")]
    FailedToFetchLog,

    #[error("waiting log timed out")]
    WaitingLogTimedOut,

    #[error("rpc error")]
    RpcError,
}

impl Reject for Error {}

impl Error {
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::KeyringItemNotFound { .. } | Self::IdNotFound { .. } => StatusCode::NOT_FOUND,
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::WaitingLogTimedOut => StatusCode::REQUEST_TIMEOUT,
            _ => StatusCode::BAD_REQUEST,
        }
    }
}
