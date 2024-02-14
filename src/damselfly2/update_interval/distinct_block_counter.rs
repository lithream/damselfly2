use rust_lapper::Lapper;
use crate::damselfly2::memory::memory_update::MemoryUpdateType;
use crate::damselfly2::update_interval::update_interval_factory::UpdateIntervalFactory;
use crate::damselfly2::update_interval::update_queue_compressor::UpdateQueueCompressor;

pub struct DistinctBlockCounter {
    memory_updates: Vec<MemoryUpdateType>,
}

impl DistinctBlockCounter {
    pub fn default() -> DistinctBlockCounter {
        DistinctBlockCounter {
            memory_updates: Vec::new(),
        }
    }

    pub fn new(memory_updates: Vec<MemoryUpdateType>) -> DistinctBlockCounter {
        DistinctBlockCounter {
            memory_updates
        }
    }

    pub fn push_update(&mut self, update: &MemoryUpdateType) {
        self.memory_updates.push(update.clone());
    }

    pub fn get_distinct_blocks(&self) -> usize {
        let compressed_updates = UpdateQueueCompressor::compress_to_allocs_only(&self.memory_updates);
        let interval_factory = UpdateIntervalFactory::new(compressed_updates);
        let mut lapper: Lapper<usize, MemoryUpdateType> = Lapper::new(interval_factory.construct_enum_vector());
        lapper.merge_overlaps();
        lapper.cov()
    }
}
