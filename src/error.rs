use multiaddr::Multiaddr;
use serde_derive::Serialize;
use thiserror::Error;
use utoipa::ToSchema;
use warp::reject::Reject;

#[derive(Debug, Clone, Error, Serialize, ToSchema)]
#[serde(rename_all = "snake_case", tag = "message")]
pub enum Error {
    #[error("invalid name: {name}")]
    InvalidName { name: String },

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
    CertNotFound { id: String },

    #[error("no backend servers")]
    EmptyBackendServers,

    #[error("port name not found: {name}")]
    NameNotFound { name: String },

    #[error("port name already exists: {name}")]
    NameAlreadyExists { name: String },
}

impl Reject for Error {}
