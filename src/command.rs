use crate::keyring::KeyringItem;
use crate::server::rpc::ErasedRpcMethod;

pub enum ServerCommand {
    AddKeyringItem {
        item: KeyringItem,
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
            ServerCommand::AddKeyringItem { item } => f
                .debug_struct("AddKeyringItem")
                .field("item", item)
                .finish(),
            ServerCommand::StopHttpChallenges => f.debug_struct("StopHttpChallenges").finish(),
            ServerCommand::CallMethod { id, .. } => {
                f.debug_struct("CallMethod").field("id", id).finish()
            }
        }
    }
}
