use std::cmp::max;
use crate::damselfly2::memory::memory_update::{MemoryUpdate, MemoryUpdateType};
use crate::damselfly2::memory::memory_usage::MemoryUsage;
use crate::damselfly2::update_interval::distinct_block_counter::DistinctBlockCounter;

pub struct MemoryUsageFactory {
    max_usage: i128,
    memory_updates: Vec<MemoryUpdateType>,
}

impl MemoryUsageFactory {
    pub fn default() -> MemoryUsageFactory {
        MemoryUsageFactory {
            max_usage: 0,
            memory_updates: Vec::new(),
        }
    }

    pub fn new(memory_updates: Vec<MemoryUpdateType>) -> MemoryUsageFactory {
        MemoryUsageFactory {
            max_usage: 0,
            memory_updates,
        }
    }

    pub fn load_memory_updates(&mut self, updates: Vec<MemoryUpdateType>) {
        self.memory_updates = updates;
    }

    pub fn calculate_usage_stats(&self) -> (Vec<MemoryUsage>, i128) {
        let mut current_usage = 0;
        let mut max_usage = 0;
        let mut memory_usages = Vec::new();
        let mut distinct_block_counter = DistinctBlockCounter::default();
        for (index, update) in self.memory_updates.iter().enumerate() {
            current_usage += Self::get_total_usage_delta(update);
            max_usage = max(max_usage, current_usage);

            distinct_block_counter.push_update(&update);
            let distinct_blocks = distinct_block_counter.get_distinct_blocks();

            memory_usages.push(MemoryUsage::new(current_usage, distinct_blocks, index));
        }
        (memory_usages, max_usage)
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