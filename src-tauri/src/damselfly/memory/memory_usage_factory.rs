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
    distinct_block_left_padding: usize,
    distinct_block_right_padding: usize,
    lapper: Lapper<usize, MemoryUpdateType>,
    counter: u64,
}

impl MemoryUsageFactory {
    pub fn new(memory_updates: Vec<MemoryUpdateType>, distinct_block_left_padding: usize, distinct_block_right_padding: usize,
                pool_start: usize, pool_stop: usize)
        -> MemoryUsageFactory {
        MemoryUsageFactory {
            memory_updates,
            lowest_address: pool_start,
            highest_address: pool_stop,
            distinct_block_left_padding,
            distinct_block_right_padding,
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
        let mut max_free_segment_fragmentation = 0;
        let mut max_largest_free_block = 0;
        let mut memory_usages = Vec::new();

        let mut distinct_block_counter = DistinctBlockCounter::new(vec![], self.distinct_block_left_padding, self.distinct_block_right_padding, Some((self.lowest_address, self.highest_address)));
        let mut max_distinct_blocks: u128 = 0;

        for (index, update) in self.memory_updates.iter().enumerate() {
            println!("Processing usage stats: {}", update.cyan());
            current_usage += Self::get_total_usage_delta(update);
            max_usage = max(max_usage, current_usage);
            distinct_block_counter.push_update(update);
            let distinct_blocks = distinct_block_counter.get_distinct_blocks();
            let free_blocks = distinct_block_counter.get_free_blocks();
            let largest_free_block = distinct_block_counter.get_largest_free_block();
            let free_segment_fragmentation = distinct_block_counter.get_free_segment_fragmentation();
            let real_timestamp_microseconds = Utility::convert_to_microseconds(update.get_real_timestamp());
            max_distinct_blocks = max(max_distinct_blocks, distinct_blocks);
            max_free_blocks = max(max_free_blocks, free_blocks.len() as u128);
            max_free_segment_fragmentation = max(max_free_segment_fragmentation, free_segment_fragmentation);
            max_largest_free_block = max(max_largest_free_block, largest_free_block.2);

            memory_usages.push(MemoryUsage::new(current_usage, distinct_blocks, largest_free_block, free_blocks.len(), free_segment_fragmentation, index, real_timestamp_microseconds, self.counter));
            self.counter += 1;
        }
        MemoryUsageStats::new(memory_usages, max_usage, max_free_blocks, max_distinct_blocks,
                              max_free_segment_fragmentation, max_largest_free_block as u128)
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
    use std::sync::Arc;
    use crate::damselfly::memory::memory_parsers::MemorySysTraceParser;
    use crate::damselfly::consts::{TEST_BINARY_PATH, TEST_LOG};
    use crate::damselfly::memory::memory_update::{Allocation, MemoryUpdateType};
    use crate::damselfly::memory::memory_usage_factory::MemoryUsageFactory;
    use crate::damselfly::memory::memory_usage_stats::MemoryUsageStats;

    fn initialise_test_log() -> MemoryUsageStats {
        let mst_parser = MemorySysTraceParser::new();
        let updates = mst_parser.parse_log_directly(TEST_LOG, TEST_BINARY_PATH).memory_updates;
        let mut memory_usage_factory = MemoryUsageFactory::new(updates, 0, 0, usize::MIN, usize::MAX);
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

    #[test]
    fn calculate_fragmentation_zero_padding_test() {
        let first_update = MemoryUpdateType::Allocation(Allocation::new(0, 8, Arc::new(String::new()), 0, String::from("0001.676 s")));
        let second_update = MemoryUpdateType::Allocation(Allocation::new(12, 8, Arc::new(String::new()), 0, String::from("0001.677 s")));
        let usage_stats =
            MemoryUsageFactory::new(vec![first_update, second_update], 0, 0, usize::MIN, usize::MAX)
                .calculate_usage_stats();
        assert_eq!(usage_stats.get_max_distinct_blocks(), 2);
    }

    #[test]
    fn calculate_fragmentation_right_padding_test() {
        let first_update = MemoryUpdateType::Allocation(Allocation::new(0, 8, Arc::new(String::new()), 0, String::from("0001.676 s")));
        let second_update = MemoryUpdateType::Allocation(Allocation::new(12, 8, Arc::new(String::new()), 0, String::from("0001.677 s")));
        let usage_stats =
            MemoryUsageFactory::new(vec![first_update, second_update], 0, 4, usize::MIN, usize::MAX)
                .calculate_usage_stats();
        assert_eq!(usage_stats.get_max_distinct_blocks(), 1);
    }

    #[test]
    fn calculate_fragmentation_left_padding_test() {
        let first_update = MemoryUpdateType::Allocation(Allocation::new(0, 8, Arc::new(String::new()), 0, String::from("0001.676 s")));
        let second_update = MemoryUpdateType::Allocation(Allocation::new(12, 8, Arc::new(String::new()), 0, String::from("0001.677 s")));
        let usage_stats =
            MemoryUsageFactory::new(vec![first_update, second_update], 0, 8, usize::MIN, usize::MAX)
                .calculate_usage_stats();
        assert_eq!(usage_stats.get_max_distinct_blocks(), 2);
    }

    #[test]
    fn calculate_fragmentation_both_padding_test() {
        let first_update = MemoryUpdateType::Allocation(Allocation::new(8, 8, Arc::new(String::new()), 0, String::from("0001.676 s")));
        let second_update = MemoryUpdateType::Allocation(Allocation::new(20, 8, Arc::new(String::new()), 1, String::from("0001.677 s")));
        let third_update = MemoryUpdateType::Allocation(Allocation::new(32, 8, Arc::new(String::new()), 2, String::from("0001.678 s")));
        let usage_stats =
            MemoryUsageFactory::new(vec![first_update.clone(), second_update.clone(), third_update.clone()], 2, 2, usize::MIN, usize::MAX)
                .calculate_usage_stats();
        assert_eq!(usage_stats.get_max_distinct_blocks(), 1);

        let usage_stats =
            MemoryUsageFactory::new(vec![first_update, second_update, third_update], 4, 0, usize::MIN, usize::MAX)
                .calculate_usage_stats();
        assert_eq!(usage_stats.get_max_distinct_blocks(), 1);
    }
}