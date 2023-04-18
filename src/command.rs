use crate::config::AppConfig;
use crate::proxy::PortContext;

#[derive(Debug)]
pub enum ServerCommand {
    SetAppConfig { config: AppConfig },
    SetPort { ctx: PortContext },
    DeletePort { name: String },
}
