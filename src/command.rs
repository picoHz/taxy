use crate::config::AppConfig;
use crate::keyring::KeyringItem;
use crate::proxy::PortContext;
use std::any::Any;

#[derive(Debug)]
pub enum ServerCommand {
    SetAppConfig {
        config: AppConfig,
    },
    SetPort {
        ctx: PortContext,
    },
    DeletePort {
        id: String,
    },
    AddKeyringItem {
        item: KeyringItem,
    },
    DeleteKeyringItem {
        id: String,
    },
    StopHttpChallenges,
    CallMethod {
        id: usize,
        method: String,
        arg: Box<dyn Any + Send + Sync>,
    },
}
