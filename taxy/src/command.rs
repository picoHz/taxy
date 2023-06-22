use crate::{certs::Cert, server::rpc::ErasedRpcMethod};
use std::sync::Arc;

pub enum ServerCommand {
    AddServerCert {
        cert: Arc<Cert>,
    },
    StopHttpChallenges,
    CallMethod {
        id: usize,
        arg: Box<dyn ErasedRpcMethod>,
    },
}

impl std::fmt::Debug for ServerCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AddServerCert { cert } => f
                .debug_struct("AddServerCert")
                .field("id", &cert.id())
                .finish(),
            Self::StopHttpChallenges => f.debug_struct("StopHttpChallenges").finish(),
            Self::CallMethod { id, .. } => f.debug_struct("CallMethod").field("id", id).finish(),
        }
    }
}
