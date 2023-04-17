use crate::proxy::PortContext;

#[derive(Debug)]
pub enum ServerCommand {
    SetPort { ctx: PortContext },
    DeletePort { name: String },
}
