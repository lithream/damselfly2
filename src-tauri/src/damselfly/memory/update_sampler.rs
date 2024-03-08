use std::cmp::{max, min};
use crate::damselfly::memory::memory_status::MemoryStatus;
use crate::damselfly::memory::memory_update::MemoryUpdateType;

pub struct UpdateSampler {
    updates: Vec<MemoryUpdateType>
}

impl UpdateSampler {
    pub fn new(updates: Vec<MemoryUpdateType>) -> UpdateSampler {
        UpdateSampler {
            updates
        }
    }
    
    pub fn sample(&self, tick_rate: u64) -> Vec<Vec<MemoryStatus>> {
        let mut current_update_queue = Vec::new();
        let mut min_address = usize::MAX;
        let mut max_address = usize::MIN;
        self.updates
            .iter()
            .for_each(|update| {
                min_address = min(min_address, update.get_absolute_address());
                max_address = max(max_address, update.get_absolute_address());
            });
        for (index, update) in self.updates {
            
        }
    }
}