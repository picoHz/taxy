use crate::certs::Cert;
use crate::config::AppConfig;
use crate::proxy::PortContext;

#[derive(Debug)]
pub enum ServerCommand {
    SetAppConfig { config: AppConfig },
    SetPort { ctx: PortContext },
    DeletePort { name: String },
    AddCert { cert: Cert },
    DeleteCert { id: String },
}
