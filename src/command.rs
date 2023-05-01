use crate::config::AppConfig;
use crate::keyring::KeyringItem;
use crate::proxy::PortContext;

#[derive(Debug)]
pub enum ServerCommand {
    SetAppConfig { config: AppConfig },
    SetPort { ctx: PortContext },
    DeletePort { id: String },
    AddKeyringItem { item: KeyringItem },
    DeleteKeyringItem { id: String },
    StopHttpChallenges,
}
