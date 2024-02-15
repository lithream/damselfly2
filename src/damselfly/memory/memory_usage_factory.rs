use std::cmp::max;
use crate::damselfly::memory::memory_update::{MemoryUpdate, MemoryUpdateType};
use crate::damselfly::memory::memory_usage::MemoryUsage;
use crate::damselfly::update_interval::distinct_block_counter::DistinctBlockCounter;

#[derive(Default)]
pub struct MemoryUsageFactory {
    memory_updates: Vec<MemoryUpdateType>,
}

impl MemoryUsageFactory {
    pub fn new(memory_updates: Vec<MemoryUpdateType>) -> MemoryUsageFactory {
        MemoryUsageFactory {
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

            distinct_block_counter.push_update(update);
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
                -(free.get_absolute_size() as i128)
            }
        }
    }
}

mod tests {
    use crate::damselfly::memory::memory_parsers::MemorySysTraceParser;
    use crate::damselfly::consts::{TEST_BINARY_PATH, TEST_LOG_PATH};
    use crate::damselfly::memory::memory_usage_factory::MemoryUsageFactory;

    #[test]
    fn calculate_usage_stats_test() {
        let mst_parser = MemorySysTraceParser::new();
        let updates = mst_parser.parse_log(TEST_LOG_PATH, TEST_BINARY_PATH);
        let memory_usage_factory = MemoryUsageFactory::new(updates);
        let (memory_usages, max_usage) = memory_usage_factory.calculate_usage_stats();
        assert_eq!(max_usage, 356);
        assert_eq!(memory_usages[0].get_memory_used_absolute(), 20);
        assert_eq!(memory_usages[1].get_memory_used_absolute(), 40);
        assert_eq!(memory_usages[2].get_memory_used_absolute(), 316);
        assert_eq!(memory_usages[3].get_memory_used_absolute(), 336);
        assert_eq!(memory_usages[4].get_memory_used_absolute(), 356);

        assert_eq!(memory_usages[0].get_distinct_blocks(), 1);
        assert_eq!(memory_usages[1].get_distinct_blocks(), 2);
        assert_eq!(memory_usages[2].get_distinct_blocks(), 3);
        assert_eq!(memory_usages[3].get_distinct_blocks(), 4);
        assert_eq!(memory_usages[4].get_distinct_blocks(), 4);
    }
}