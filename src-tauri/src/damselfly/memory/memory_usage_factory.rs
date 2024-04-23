use std::cmp::{max};
use owo_colors::OwoColorize;
use rust_lapper::Lapper;
use crate::damselfly::memory::memory_update::{MemoryUpdate, MemoryUpdateType};
use crate::damselfly::memory::memory_usage::MemoryUsage;
use crate::damselfly::memory::memory_usage_stats::MemoryUsageStats;
use crate::damselfly::memory::utility::Utility;
use crate::damselfly::update_interval::distinct_block_counter::DistinctBlockCounter;

pub struct MemoryUsageFactory {
    memory_updates: Vec<MemoryUpdateType>,
    lowest_address: usize,
    highest_address: usize,
    lapper: Lapper<usize, MemoryUpdateType>,
    counter: u64,
}

impl MemoryUsageFactory {
    pub fn new(memory_updates: Vec<MemoryUpdateType>) -> MemoryUsageFactory {
        MemoryUsageFactory {
            memory_updates,
            lowest_address: usize::MAX,
            highest_address: usize::MIN,
            lapper: Lapper::new(vec![]),
            counter: 0,
        }
    }

    pub fn load_memory_updates(&mut self, updates: Vec<MemoryUpdateType>) {
        self.memory_updates = updates;
    }

    pub fn calculate_usage_stats(&mut self) -> MemoryUsageStats {
        let mut current_usage = 0;
        let mut max_usage = 0;
        let mut max_free_blocks: u128 = 0;
        let mut memory_usages = Vec::new();

        let mut distinct_block_counter = DistinctBlockCounter::default();
        let mut max_distinct_blocks: u128 = 0;

        for (index, update) in self.memory_updates.iter().enumerate() {
            if self.counter % 1000 == 0 {
                println!("Processing usage stats: {}", update.cyan());
            }
            current_usage += Self::get_total_usage_delta(update);
            max_usage = max(max_usage, current_usage);
            distinct_block_counter.push_update(update);
            let distinct_blocks = distinct_block_counter.get_distinct_blocks();
            let free_blocks = distinct_block_counter.get_free_blocks();
            let largest_free_block = distinct_block_counter.get_largest_free_block();
            let real_timestamp_microseconds = Utility::convert_to_microseconds(&update.get_real_timestamp());
            max_distinct_blocks = max(max_distinct_blocks, distinct_blocks as u128);
            max_free_blocks = max(max_free_blocks, free_blocks.len() as u128);

            memory_usages.push(MemoryUsage::new(current_usage, distinct_blocks as usize, largest_free_block, free_blocks.len(), index, real_timestamp_microseconds));
            self.counter += 1;
        }
        MemoryUsageStats::new(memory_usages, max_usage, max_free_blocks, max_distinct_blocks)
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
    use crate::damselfly::consts::{TEST_BINARY_PATH, TEST_LOG};
    use crate::damselfly::memory::memory_usage_factory::MemoryUsageFactory;
    use crate::damselfly::memory::memory_usage_stats::MemoryUsageStats;

    fn initialise_test_log() -> MemoryUsageStats {
        let mst_parser = MemorySysTraceParser::new();
        let updates = mst_parser.parse_log_directly(TEST_LOG, TEST_BINARY_PATH).memory_updates;
        let mut memory_usage_factory = MemoryUsageFactory::new(updates);
        memory_usage_factory.calculate_usage_stats()
    }

    #[test]
    fn calculate_max_usage_test() {
        let memory_usage_stats = initialise_test_log();
        let memory_usages = memory_usage_stats.get_memory_usages();

        assert_eq!(memory_usage_stats.get_max_usage(), 356);
        assert_eq!(memory_usages[0].get_memory_used_absolute(), 20);
        assert_eq!(memory_usages[1].get_memory_used_absolute(), 40);
        assert_eq!(memory_usages[2].get_memory_used_absolute(), 316);
        assert_eq!(memory_usages[3].get_memory_used_absolute(), 336);
        assert_eq!(memory_usages[4].get_memory_used_absolute(), 356);
    }

    #[test]
    fn calculate_memory_used_absolute_test() {
        let memory_usage_stats = initialise_test_log();
        let memory_usages = memory_usage_stats.get_memory_usages();

        assert_eq!(memory_usages[0].get_distinct_blocks(), 1);
        assert_eq!(memory_usages[1].get_distinct_blocks(), 2);
        assert_eq!(memory_usages[2].get_distinct_blocks(), 3);
        assert_eq!(memory_usages[3].get_distinct_blocks(), 4);
        assert_eq!(memory_usages[4].get_distinct_blocks(), 4);
    }

    #[test]
    fn calculate_latest_operation_test() {
        let memory_usage_stats = initialise_test_log();
        let memory_usages = memory_usage_stats.get_memory_usages();

        assert_eq!(memory_usages[0].get_latest_operation(), 0);
        assert_eq!(memory_usages[1].get_latest_operation(), 1);
        assert_eq!(memory_usages[2].get_latest_operation(), 2);
        assert_eq!(memory_usages[3].get_latest_operation(), 3);
        assert_eq!(memory_usages[4].get_latest_operation(), 4);
    }
}