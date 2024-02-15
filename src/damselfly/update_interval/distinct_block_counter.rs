use std::cmp::{max, min};
use rust_lapper::Lapper;
use crate::damselfly::memory::memory_update::{MemoryUpdate, MemoryUpdateType};
use crate::damselfly::update_interval::update_interval_factory::UpdateIntervalFactory;
use crate::damselfly::update_interval::update_queue_compressor::UpdateQueueCompressor;

#[derive(Default)]
pub struct DistinctBlockCounter {
    start: usize,
    stop: usize,
    memory_updates: Vec<MemoryUpdateType>,
}

impl DistinctBlockCounter {
    pub fn new(memory_updates: Vec<MemoryUpdateType>) -> DistinctBlockCounter {
        DistinctBlockCounter {
            start: usize::MAX,
            stop: usize::MIN,
            memory_updates
        }
    }

    pub fn push_update(&mut self, update: &MemoryUpdateType) {
        self.memory_updates.push(update.clone());
        let new_start;
        let new_stop;
        match update {
            MemoryUpdateType::Allocation(allocation) => {
                new_start = allocation.get_absolute_address();
                new_stop = new_start + allocation.get_absolute_size()
            }
            MemoryUpdateType::Free(free) => {
                new_start = free.get_absolute_address();
                new_stop = new_start + free.get_absolute_size();
            }
        }
        self.start = min(self.start, new_start);
        self.stop = max(self.stop, new_stop);
    }

    pub fn get_distinct_blocks(&self) -> usize {
        let compressed_updates = UpdateQueueCompressor::compress_to_allocs_only(&self.memory_updates);
        let interval_factory = UpdateIntervalFactory::new(compressed_updates);
        let mut lapper: Lapper<usize, MemoryUpdateType> = Lapper::new(interval_factory.construct_enum_vector());
        lapper.merge_overlaps();
        lapper.find(self.start, self.stop).count()
    }
}
