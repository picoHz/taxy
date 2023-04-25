use crate::certs::Cert;
use crate::config::AppConfig;
use crate::proxy::PortContext;
use std::sync::Arc;

#[derive(Debug)]
pub enum ServerCommand {
    SetAppConfig { config: AppConfig },
    SetPort { ctx: PortContext },
    DeletePort { name: String },
    AddCert { cert: Arc<Cert> },
    DeleteCert { id: String },
}
