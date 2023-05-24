use crate::keyring::KeyringItem;
use crate::proxy::PortContext;
use crate::server::rpc::ErasedRpcMethod;

pub enum ServerCommand {
    SetPort {
        ctx: PortContext,
    },
    UpdatePorts,
    AddKeyringItem {
        item: KeyringItem,
    },
    DeleteKeyringItem {
        id: String,
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
            ServerCommand::SetPort { ctx } => {
                f.debug_struct("SetPort").field("ctx", ctx).finish()
            }
            ServerCommand::UpdatePorts => f.debug_struct("UpdatePorts").finish(),
            ServerCommand::AddKeyringItem { item } => {
                f.debug_struct("AddKeyringItem").field("item", item).finish()
            }
            ServerCommand::DeleteKeyringItem { id } => {
                f.debug_struct("DeleteKeyringItem").field("id", id).finish()
            }
            ServerCommand::StopHttpChallenges => {
                f.debug_struct("StopHttpChallenges").finish()
            }
            ServerCommand::CallMethod { id, .. } => f
                .debug_struct("CallMethod")
                .field("id", id)
                .finish(),
        }
    }
}