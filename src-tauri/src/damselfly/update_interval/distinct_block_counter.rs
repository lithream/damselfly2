use std::cmp::{max, min};
use std::time::Instant;
use rust_lapper::Lapper;
use crate::damselfly::memory::memory_update::{MemoryUpdate, MemoryUpdateType};
use crate::damselfly::memory::NoHashMap;
use crate::damselfly::update_interval::update_interval_factory::UpdateIntervalFactory;
use crate::damselfly::update_interval::update_queue_compressor::UpdateQueueCompressor;

pub struct DistinctBlockCounter {
    start: usize,
    stop: usize,
    memory_updates: NoHashMap<usize, MemoryUpdateType>,
    lapper: Lapper<usize, MemoryUpdateType>,
//    memory_updates: Vec<MemoryUpdateType>,
}

impl Default for DistinctBlockCounter {
    fn default() -> Self {
        Self {
            start: 0,
            stop: 0,
            memory_updates: NoHashMap::default(),
            lapper: Lapper::new(vec![]),
        }
    }
}
impl DistinctBlockCounter {
    pub fn new(memory_updates: Vec<MemoryUpdateType>) -> DistinctBlockCounter {
        let mut memory_updates_map: NoHashMap<usize, MemoryUpdateType> = NoHashMap::default();
        for memory_update in memory_updates {
            memory_updates_map.insert(memory_update.get_absolute_address(), memory_update);
        }
        DistinctBlockCounter {
            start: usize::MAX,
            stop: usize::MIN,
            memory_updates: memory_updates_map,
            lapper: Lapper::new(vec![]),
        }
    }

    pub fn push_update(&mut self, update: &MemoryUpdateType) {
        match update {
            MemoryUpdateType::Allocation(allocation) => {
                // skip remaking lapper as we can just insert the new interval into it
                self.memory_updates.insert(allocation.get_absolute_address(), update.clone());
                self.lapper.insert(UpdateIntervalFactory::convert_update_to_interval(update));
            }
            MemoryUpdateType::Free(free) => {
                // remake lapper as there is no way to remove intervals directly from the lapper
                self.memory_updates.remove(&free.get_absolute_address());
                self.initialise_lapper();
            }
        };
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

    fn initialise_lapper(&mut self) {
        self.lapper = Lapper::new(UpdateIntervalFactory::new(
            self.memory_updates.values().cloned().collect()).construct_enum_vector());
    }
    
    pub fn get_distinct_blocks(&mut self) -> usize {
        self.lapper.merge_overlaps();
        self.lapper.find(self.start, self.stop).count()
    }
}

mod tests {
    use crate::damselfly::consts::{TEST_BINARY_PATH, TEST_LOG};
    use crate::damselfly::memory::memory_parsers::MemorySysTraceParser;
    use crate::damselfly::memory::memory_update::MemoryUpdateType;
    use crate::damselfly::update_interval::distinct_block_counter::DistinctBlockCounter;

    fn initialise_test_log() -> (Vec<MemoryUpdateType>, DistinctBlockCounter) {
        let mst_parser = MemorySysTraceParser::new();
        let updates = mst_parser.parse_log_directly(TEST_LOG, TEST_BINARY_PATH);
        (updates, DistinctBlockCounter::default())
    }

    #[test]
    fn zero_distinct_blocks_test() {
        let (_, mut distinct_block_counter) = initialise_test_log();
        assert_eq!(distinct_block_counter.get_distinct_blocks(), 0);
    }

    #[test]
    fn one_distinct_block_test() {
        let (updates, mut distinct_block_counter) = initialise_test_log();
        distinct_block_counter.push_update(&updates[0]);
    }
}