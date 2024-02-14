use std::cmp::max;
use crate::damselfly2::memory::memory_update::{MemoryUpdate, MemoryUpdateType};
use crate::damselfly2::memory::memory_usage::MemoryUsage;

pub struct MemoryUsageFactory {
    max_usage: i128,
    usage_vec: Vec<MemoryUsage>,
}

impl MemoryUsageFactory {
    pub fn new() -> MemoryUsageFactory {
        MemoryUsageFactory {
            max_usage: 0,
            usage_vec: Vec::new(),
        }
    }

    pub fn load_memory_updates(&mut self, updates: &Vec<MemoryUpdateType>) {
        let mut current_absolute_usage = 0;
        for (index, update) in updates.iter().enumerate() {
            current_absolute_usage += Self::get_total_usage_delta(update);
            self.max_usage = max(current_absolute_usage, self.max_usage);
            let current_memory_usage =
                MemoryUsage::new(current_absolute_usage as usize,
        }
    }

    fn get_total_usage_delta(memory_update: &MemoryUpdateType) -> i128 {
        match memory_update {
            MemoryUpdateType::Allocation(allocation) => {
                allocation.get_absolute_size() as i128
            }
            MemoryUpdateType::Free(free) => {
                free.get_absolute_size() as i128 * -1
            }
        }
    }
}