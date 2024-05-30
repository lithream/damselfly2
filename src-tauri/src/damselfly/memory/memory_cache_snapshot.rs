use crate::damselfly::memory::memory_status::MemoryStatus;
use crate::damselfly::update_interval::UpdateInterval;
use crate::damselfly::viewer::memory_canvas::MemoryCanvas;

pub struct MemoryCacheSnapshot {
    base: MemoryCanvas,
    temporary_updates: Vec<UpdateInterval>
}

impl MemoryCacheSnapshot {
    pub fn new(base: MemoryCanvas, temporary_updates: Vec<UpdateInterval>) -> Self {
        Self {
            base,
            temporary_updates,
        }
    }
    pub fn render_at(&self, time: usize) -> Vec<MemoryStatus> {
        let mut updates_to_append = Vec::new();
        for update in &self.temporary_updates {
            if update.val.get_timestamp() > time {
                break;
            }
            updates_to_append.push(update.clone());
        }
        self.base.render_temporary(updates_to_append)
    }
    
    pub fn get_base(&self) -> &MemoryCanvas {
        &self.base
    }
}