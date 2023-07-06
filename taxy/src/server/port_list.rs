use crate::proxy::PortContext;
use taxy_api::port::PortEntry;

#[derive(Default)]
pub struct PortList {
    contexts: Vec<PortContext>,
}

impl PortList {
    pub fn entries(&self) -> impl Iterator<Item = &PortEntry> {
        self.contexts.iter().map(|c| c.entry())
    }

    pub fn as_slice(&self) -> &[PortContext] {
        &self.contexts
    }

    pub fn as_mut_slice(&mut self) -> &mut [PortContext] {
        &mut self.contexts
    }

    pub fn get(&self, id: &str) -> Option<&PortContext> {
        self.contexts.iter().find(|p| p.entry().id == *id)
    }

    pub fn update(&mut self, ctx: PortContext) -> bool {
        if let Some(index) = self
            .contexts
            .iter()
            .position(|p| p.entry().id == ctx.entry().id)
        {
            if self.contexts[index].entry != ctx.entry {
                self.contexts[index].apply(ctx);
                true
            } else {
                false
            }
        } else {
            self.contexts.push(ctx);
            true
        }
    }

    pub fn delete(&mut self, id: &str) -> bool {
        if let Some(index) = self.contexts.iter().position(|p| p.entry().id == *id) {
            self.contexts.remove(index).reset();
            true
        } else {
            false
        }
    }

    pub fn reset(&mut self, id: &str) -> bool {
        if let Some(index) = self.contexts.iter().position(|p| p.entry().id == *id) {
            self.contexts[index].reset();
            true
        } else {
            false
        }
    }
}
