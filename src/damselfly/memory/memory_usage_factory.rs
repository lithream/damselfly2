use std::cmp::{max, min};
use rust_lapper::Lapper;
use crate::damselfly::memory::memory_update::{MemoryUpdate, MemoryUpdateType};
use crate::damselfly::memory::memory_usage::MemoryUsage;
use crate::damselfly::update_interval::distinct_block_counter::DistinctBlockCounter;
use crate::damselfly::update_interval::update_interval_factory::UpdateIntervalFactory;
use crate::damselfly::update_interval::UpdateInterval;

pub struct MemoryUsageFactory {
    memory_updates: Vec<MemoryUpdateType>,
    lowest_address: usize,
    highest_address: usize,
    lapper: Lapper<usize, MemoryUpdateType>,
}

impl MemoryUsageFactory {
    pub fn new(memory_updates: Vec<MemoryUpdateType>) -> MemoryUsageFactory {
        MemoryUsageFactory {
            memory_updates,
            lowest_address: usize::MAX,
            highest_address: usize::MIN,
            lapper: Lapper::new(vec![]),
        }
    }

    pub fn load_memory_updates(&mut self, updates: Vec<MemoryUpdateType>) {
        self.memory_updates = updates;
    }

    pub fn calculate_usage_stats(&mut self) -> (Vec<MemoryUsage>, i128, usize) {
        let mut current_usage = 0;
        let mut max_usage = 0;
        let mut memory_usages = Vec::new();

        let mut distinct_block_counter = DistinctBlockCounter::default();
        let mut max_distinct_blocks = 0;

        for (index, update) in self.memory_updates.iter().enumerate() {
            eprintln!("enter {index}");
            let (largest_free_block_size, free_blocks) =
                Self::calculate_new_free_blocks(&mut self.lowest_address, &mut self.highest_address, &mut self.lapper, update.clone());
            current_usage += Self::get_total_usage_delta(update);
            max_usage = max(max_usage, current_usage);

            eprintln!("exit {index}");
            distinct_block_counter.push_update(update);
            let distinct_blocks = distinct_block_counter.get_distinct_blocks();
            max_distinct_blocks = max(max_distinct_blocks, distinct_blocks);

            memory_usages.push(MemoryUsage::new(current_usage, distinct_blocks, index, largest_free_block_size, free_blocks));
        }
        (memory_usages, max_usage, max_distinct_blocks)
    }

    fn calculate_new_free_blocks(cur_lowest_address: &mut usize, cur_highest_address: &mut usize, lapper: &mut Lapper<usize, MemoryUpdateType>, memory_update: MemoryUpdateType) -> (usize, usize) {
        *cur_lowest_address = min(*cur_lowest_address, memory_update.get_absolute_address());
        *cur_highest_address = max(*cur_highest_address, memory_update.get_absolute_address());
        lapper.insert(UpdateIntervalFactory::convert_update_to_interval(&memory_update));

        let mut largest_free_block_size = 0;
        let mut free_blocks = 0;
        let mut left = *cur_lowest_address;
        let mut right = left + 1;

        while right < *cur_highest_address {
            while lapper.find(left, right).count() == 0 {
                right += 1;
            }
            largest_free_block_size = max(largest_free_block_size, right - left);
            free_blocks += 1;
            left = right;
            right = left + 1;
        }
        (largest_free_block_size, free_blocks)
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
    use crate::damselfly::consts::{TEST_BINARY_PATH, TEST_LOG, TEST_LOG_PATH};
    use crate::damselfly::memory::memory_usage::MemoryUsage;
    use crate::damselfly::memory::memory_usage_factory::MemoryUsageFactory;

    fn initialise_test_log() -> (Vec<MemoryUsage>, i128, usize) {
        let mst_parser = MemorySysTraceParser::new();
        let updates = mst_parser.parse_log_directly(TEST_LOG, TEST_BINARY_PATH);
        let mut memory_usage_factory = MemoryUsageFactory::new(updates);
        memory_usage_factory.calculate_usage_stats()
    }

    #[test]
    fn calculate_max_usage_test() {
        let (memory_usages, max_usage, _) = initialise_test_log();

        assert_eq!(max_usage, 356);
        assert_eq!(memory_usages[0].get_memory_used_absolute(), 20);
        assert_eq!(memory_usages[1].get_memory_used_absolute(), 40);
        assert_eq!(memory_usages[2].get_memory_used_absolute(), 316);
        assert_eq!(memory_usages[3].get_memory_used_absolute(), 336);
        assert_eq!(memory_usages[4].get_memory_used_absolute(), 356);
    }

    #[test]
    fn calculate_memory_used_absolute_test() {
        let (memory_usages, _, _) = initialise_test_log();

        assert_eq!(memory_usages[0].get_distinct_blocks(), 1);
        assert_eq!(memory_usages[1].get_distinct_blocks(), 2);
        assert_eq!(memory_usages[2].get_distinct_blocks(), 3);
        assert_eq!(memory_usages[3].get_distinct_blocks(), 4);
        assert_eq!(memory_usages[4].get_distinct_blocks(), 4);
    }

    #[test]
    fn calculate_latest_operation_test() {
        let (memory_usages, _, _) = initialise_test_log();

        assert_eq!(memory_usages[0].get_latest_operation(), 0);
        assert_eq!(memory_usages[1].get_latest_operation(), 1);
        assert_eq!(memory_usages[2].get_latest_operation(), 2);
        assert_eq!(memory_usages[3].get_latest_operation(), 3);
        assert_eq!(memory_usages[4].get_latest_operation(), 4);
    }
}