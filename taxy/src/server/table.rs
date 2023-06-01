use crate::{config::port::PortEntry, proxy::PortContext};

pub struct ProxyTable {
    contexts: Vec<PortContext>,
}

impl ProxyTable {
    pub fn new() -> Self {
        Self {
            contexts: Vec::new(),
        }
    }

    pub fn entries(&self) -> Vec<PortEntry> {
        self.contexts.iter().map(|c| c.entry().clone()).collect()
    }

    pub fn contexts(&self) -> &[PortContext] {
        &self.contexts
    }

    pub fn contexts_mut(&mut self) -> &mut [PortContext] {
        &mut self.contexts
    }

    pub fn set_port(&mut self, ctx: PortContext) {
        if let Some(index) = self
            .contexts
            .iter()
            .position(|p| p.entry().id == ctx.entry().id)
        {
            self.contexts[index].apply(ctx);
        } else {
            self.contexts.push(ctx);
        }
    }

    pub fn delete_port(&mut self, id: &str) -> bool {
        if let Some(index) = self.contexts.iter().position(|p| p.entry().id == *id) {
            self.contexts.remove(index).reset();
            true
        } else {
            false
        }
    }

    pub fn reset_port(&mut self, id: &str) -> bool {
        if let Some(index) = self.contexts.iter().position(|p| p.entry().id == *id) {
            self.contexts[index].reset();
            true
        } else {
            false
        }
    }
}
