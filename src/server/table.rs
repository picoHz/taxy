use crate::{config::port::PortEntry, proxy::PortContext};
use tracing::error;

pub struct ProxyTable {
    entries: Vec<PortEntry>,
    contexts: Vec<PortContext>,
}

impl ProxyTable {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            contexts: Vec::new(),
        }
    }

    pub fn entries(&self) -> &[PortEntry] {
        &self.entries
    }

    pub fn contexts(&self) -> &[PortContext] {
        &self.contexts
    }

    pub fn contexts_mut(&mut self) -> &mut [PortContext] {
        &mut self.contexts
    }

    pub fn set_port(&mut self, name: &str, port: PortEntry) {
        let ctx = match PortContext::new(&port) {
            Ok(ctx) => ctx,
            Err(err) => {
                error!(?err, "failed to create proxy state");
                return;
            }
        };
        if let Some(index) = self.entries.iter().position(|p| p.name == name) {
            self.entries[index] = port;
            self.contexts[index].apply(ctx);
        } else {
            self.entries.push(port);
            self.contexts.push(ctx);
        }
    }

    pub fn delete_port(&mut self, name: &str) {
        if let Some(index) = self.entries.iter().position(|p| p.name == *name) {
            self.entries.remove(index);
            self.contexts.remove(index);
        }
    }
}
