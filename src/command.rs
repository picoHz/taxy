use std::collections::HashMap;

use crate::config::AppConfig;
use crate::keyring::KeyringItem;
use crate::proxy::PortContext;

#[derive(Debug)]
pub enum ServerCommand {
    SetAppConfig { config: AppConfig },
    SetPort { ctx: PortContext },
    DeletePort { name: String },
    AddKeyringItem { item: KeyringItem },
    DeleteKeyringItem { id: String },
    SetHttpChallenges { challenges: HashMap<String, String> },
}
