use std::cmp::{max, min};
use std::collections::HashSet;
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
    starts: HashSet<usize>,
    ends: HashSet<usize>,
    distinct_blocks: i64,
}

impl Default for DistinctBlockCounter {
    fn default() -> Self {
        Self {
            start: 0,
            stop: 0,
            memory_updates: NoHashMap::default(),
            lapper: Lapper::new(vec![]),
            starts: HashSet::new(),
            ends: HashSet::new(),
            distinct_blocks: 0,
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
            starts: HashSet::new(),
            ends: HashSet::new(),
            distinct_blocks: 0,
        }
    }

    pub fn push_update(&mut self, update: &MemoryUpdateType) {
        let start = update.get_start();
        let end = update.get_end();
        let mut left_attached = false;
        let mut right_attached = false;
        let mut block_delta: i64 = 0;
        
        if self.ends.contains(&start) {
            left_attached = true;
        }
        if self.starts.contains(&end) {
            right_attached = true;
        }

        
        match update {
            MemoryUpdateType::Allocation(allocation) => {
                // glues together two blocks, reducing fragmentation
                if left_attached && right_attached {
                    block_delta = -1;
                }

                // island block with no blocks surrounding it, increasing fragmentation
                if !left_attached && !right_attached {
                    block_delta = 1;
                }

                // otherwise, glues onto an existing block, leaving fragmentation unchanged
                self.starts.insert(start);
                self.ends.insert(end);
            }
            MemoryUpdateType::Free(free) => {
                // breaks a block into two blocks, increasing fragmentation
                if left_attached && right_attached {
                    block_delta = 1;
                }
                
                // frees an island block, reducing fragmentation
                if !left_attached && !right_attached {
                    block_delta = -1;
                }
                
                // otherwise, frees a block glued onto another, leaving fragmentation unchanged
                self.starts.remove(&start);
                self.ends.remove(&end);
            }
        };
        
        self.calculate_new_memory_bounds(update);
        self.distinct_blocks = self.distinct_blocks.saturating_add(block_delta);
    }
    
    fn calculate_new_memory_bounds(&mut self, update: &MemoryUpdateType) {
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
    
    pub fn get_distinct_blocks(&mut self) -> i64 {
        self.distinct_blocks
        /*
        self.lapper.merge_overlaps();
        self.lapper.find(self.start, self.stop).count()
        
         */
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