use multiaddr::Multiaddr;
use serde_derive::Serialize;
use thiserror::Error;
use warp::reject::Reject;

#[derive(Debug, Clone, Error, Serialize)]
#[serde(rename_all = "snake_case", tag = "message")]
pub enum Error {
    #[error("invalid name: {name}")]
    InvalidName { name: String },

    #[error("invalid listening address: {addr}")]
    InvalidListeningAddress { addr: Multiaddr },

    #[error("invalid server address: {addr}")]
    InvalidServerAddress { addr: Multiaddr },

    #[error("no backend servers")]
    EmptyBackendServers,

    #[error("port name not found: {name}")]
    NameNotFound { name: String },

    #[error("port name already exists: {name}")]
    NameAlreadyExists { name: String },
}

impl Reject for Error {}
