use crate::config::port::PortEntry;

#[derive(Debug)]
pub enum ServerCommand {
    SetPort { name: String, entry: PortEntry },
    DeletePort { name: String },
}
