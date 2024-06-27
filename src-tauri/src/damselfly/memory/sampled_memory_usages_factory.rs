use std::cmp::{max, min};
use std::collections::HashMap;

use crate::damselfly::memory::memory_usage::MemoryUsage;
use crate::damselfly::memory::memory_usage_sample::MemoryUsageSample;
use crate::damselfly::memory::utility::Utility;

pub struct SampledMemoryUsagesFactory {
    current_time: u64,
    sample_interval: u64,
    memory_usages: Vec<MemoryUsage>,
}

impl SampledMemoryUsagesFactory {
    pub fn new(sample_interval: u64, memory_usages: Vec<MemoryUsage>) -> Self {
        Self {
            current_time: 0,
            sample_interval,
            memory_usages,
        }
    }

    // average usages in each bucket
    // push into vec
    // nonexistent timestamps should just dupe the previous one
    pub fn divide_usages_into_buckets(&self) -> Vec<MemoryUsageSample> {
        let mut buckets = HashMap::new();
        for usage in &self.memory_usages {
            let rounded_timestamp = Utility::round_to_nearest_multiple_of(usage.get_timestamp_microseconds(), self.sample_interval);
            buckets.entry(rounded_timestamp).or_insert(Vec::new()).push(usage.clone());
        }

        let mut averaged_buckets = Vec::new();
        let mut bucket_keys: Vec<u64> = buckets.keys().cloned().collect();
        bucket_keys.sort();
        let last_key = *bucket_keys.last().unwrap_or(&0);
        let mut previous_averaged_usage = MemoryUsage::new(0, 0, (0, 0, 0), 0, 0, 0, 0, 0);
        let mut previous_first_last_operations = (u64::MAX, u64::MIN);
        for key in (0..=last_key).step_by(self.sample_interval as usize) {
            match buckets.get(&key) {
                None => {
                    averaged_buckets.push(MemoryUsageSample::new(Vec::new(), previous_first_last_operations.0, previous_first_last_operations.1, previous_averaged_usage.clone()));
                }
                Some(usages) => {
                    let mut bucket_usage = 
                        MemoryUsage::new(
                            0,
                            0,
                            (0, 0, 0),
                            0,
                            0,
                            key as usize,
                            0,
                            0,
                        );
                    let mut bucket_memory_used = 0;
                    let mut bucket_distinct_blocks: u128 = 0;
                    let mut bucket_largest_free_block = (0, 0, 0);
                    let mut bucket_free_blocks = 0;
                    let mut bucket_free_segment_fragmentation = 0;
                    let mut bucket_latest_operation = 0;
                    let mut bucket_timestamp = 0;
                    let mut first_last_operations: (u64, u64) = (u64::MAX, u64::MIN);

                    for usage in usages {
                        first_last_operations.0 = min(usage.get_latest_operation() as u64, first_last_operations.0);
                        first_last_operations.1 = max(usage.get_latest_operation() as u64, first_last_operations.1);

                        bucket_memory_used += usage.get_memory_used_absolute();
                        bucket_distinct_blocks += usage.get_distinct_blocks();
                        if usage.get_largest_free_block().2 > bucket_largest_free_block.2 {
                            bucket_largest_free_block = usage.get_largest_free_block();
                        }
                        bucket_free_blocks += usage.get_free_blocks();
                        bucket_free_segment_fragmentation += usage.get_free_segment_fragmentation();
                        bucket_latest_operation = usage.get_latest_operation();
                        bucket_timestamp = usage.get_timestamp();
                    }
                    
                    bucket_usage.set_memory_used_absolute(bucket_memory_used / usages.len() as i128);
                    bucket_usage.set_distinct_blocks(bucket_distinct_blocks / usages.len() as u128);
                    bucket_usage.set_largest_free_block(bucket_largest_free_block);
                    bucket_usage.set_free_blocks(bucket_free_blocks / usages.len());
                    bucket_usage.set_free_segment_fragmentation(bucket_free_segment_fragmentation / usages.len() as u128);
                    bucket_usage.set_latest_operation(bucket_latest_operation);
                    bucket_usage.set_timestamp(bucket_timestamp);
                    previous_averaged_usage = bucket_usage.clone();
                    previous_first_last_operations = first_last_operations;
                    averaged_buckets.push(MemoryUsageSample::new(usages.clone(), first_last_operations.0, first_last_operations.1, bucket_usage.clone()));
                }
            }
        }
        averaged_buckets
    }

    pub fn get_sampled_memory_usages(&self) {

    }
}

mod tests {
    use crate::damselfly::memory::memory_usage::MemoryUsage;
    use crate::damselfly::memory::sampled_memory_usages_factory::SampledMemoryUsagesFactory;

    #[test]
    fn sample_no_updates() {
        let memory_usage_sampler = SampledMemoryUsagesFactory::new(1, Vec::new());
        assert!(memory_usage_sampler.divide_usages_into_buckets().is_empty());
    }
    
    #[test]
    fn sample_one_update() {
        let memory_usages = vec![
            MemoryUsage::new(1, 1, (0, 0, 0), 1, 0, 1, 0, 0),
        ];
        let memory_usage_sampler = SampledMemoryUsagesFactory::new(1, memory_usages);
        assert_eq!(memory_usage_sampler.divide_usages_into_buckets().first().unwrap().get_first(), 1);
        assert_eq!(memory_usage_sampler.divide_usages_into_buckets().first().unwrap().get_last(), 1);
        assert_eq!(memory_usage_sampler.divide_usages_into_buckets().first().unwrap().get_sampled_usage().get_memory_used_absolute(), 1);
        assert_eq!(memory_usage_sampler.divide_usages_into_buckets().first().unwrap().get_sampled_usage().get_distinct_blocks(), 1);
        assert_eq!(memory_usage_sampler.divide_usages_into_buckets().first().unwrap().get_sampled_usage().get_free_blocks(), 1);
        assert_eq!(memory_usage_sampler.divide_usages_into_buckets().first().unwrap().get_sampled_usage().get_largest_free_block(), (0, 0, 0));
        assert_eq!(memory_usage_sampler.divide_usages_into_buckets().first().unwrap().get_sampled_usage().get_free_segment_fragmentation(), 0);
        assert_eq!(memory_usage_sampler.divide_usages_into_buckets().first().unwrap().get_sampled_usage().get_latest_operation(), 1);
        assert_eq!(memory_usage_sampler.divide_usages_into_buckets().first().unwrap().get_sampled_usage().get_timestamp_microseconds(), 0);
    }

    #[test]
    fn sample_multiple_updates_no_overlap() {
        let memory_usages = vec![
            MemoryUsage::new(1, 1, (1, 2, 1), 1, 0, 1, 1, 0),
            MemoryUsage::new(2, 2, (2, 4, 2), 2, 0, 2, 2, 0),
            MemoryUsage::new(3, 3, (3, 6, 3), 3, 0, 3, 3, 0),
        ];
        let memory_usage_sampler = SampledMemoryUsagesFactory::new(1, memory_usages);
        let buckets = memory_usage_sampler.divide_usages_into_buckets();
        assert_eq!(buckets[0].get_sampled_usage().get_memory_used_absolute(), 0);
        assert_eq!(buckets[0].get_sampled_usage().get_distinct_blocks(), 0);
        assert_eq!(buckets[0].get_sampled_usage().get_free_blocks(), 0);
        assert_eq!(buckets[0].get_sampled_usage().get_largest_free_block(), (0, 0, 0));
        assert_eq!(buckets[0].get_sampled_usage().get_latest_operation(), 0);
        assert_eq!(buckets[0].get_sampled_usage().get_timestamp_microseconds(), 0);
        assert_eq!(buckets[1].get_sampled_usage().get_memory_used_absolute(), 1);
        assert_eq!(buckets[1].get_sampled_usage().get_distinct_blocks(), 1);
        assert_eq!(buckets[1].get_sampled_usage().get_free_blocks(), 1);
        assert_eq!(buckets[1].get_sampled_usage().get_largest_free_block(), (1, 2, 1));
        assert_eq!(buckets[1].get_sampled_usage().get_latest_operation(), 1);
        assert_eq!(buckets[1].get_sampled_usage().get_timestamp_microseconds(), 1);
        assert_eq!(buckets[2].get_sampled_usage().get_memory_used_absolute(), 2);
        assert_eq!(buckets[2].get_sampled_usage().get_distinct_blocks(), 2);
        assert_eq!(buckets[2].get_sampled_usage().get_free_blocks(), 2);
        assert_eq!(buckets[2].get_sampled_usage().get_largest_free_block(), (2, 4, 2));
        assert_eq!(buckets[2].get_sampled_usage().get_latest_operation(), 2);
        assert_eq!(buckets[2].get_sampled_usage().get_timestamp_microseconds(), 2);
        assert_eq!(buckets[3].get_sampled_usage().get_memory_used_absolute(), 3);
        assert_eq!(buckets[3].get_sampled_usage().get_distinct_blocks(), 3);
        assert_eq!(buckets[3].get_sampled_usage().get_free_blocks(), 3);
        assert_eq!(buckets[3].get_sampled_usage().get_largest_free_block(), (3, 6, 3));
        assert_eq!(buckets[3].get_sampled_usage().get_latest_operation(), 3);
        assert_eq!(buckets[3].get_sampled_usage().get_timestamp_microseconds(), 3);
    }
    
    #[test]
    fn sample_multiple_updates_overlap() {
        let memory_usages = vec![
            // timestamp = 1
            MemoryUsage::new(1, 1, (1, 2, 1), 1, 0, 1, 1, 0),
            MemoryUsage::new(2, 2, (2, 4, 2), 2, 0, 2, 1, 0),
            MemoryUsage::new(3, 3, (3, 6, 3), 3, 0, 3, 1, 0),

            // timestamp = 2
            MemoryUsage::new(4, 4, (4, 8, 4), 4, 0,4, 2, 0),
            MemoryUsage::new(5, 5, (5, 10, 5), 5, 0,5, 2, 0),

            // timestamp = 3
            MemoryUsage::new(6, 6, (6, 12, 6), 6, 0, 6, 3, 0),
        ];
        
        let memory_usage_sampler = SampledMemoryUsagesFactory::new(1, memory_usages);
        let buckets = memory_usage_sampler.divide_usages_into_buckets();       
        
        assert_eq!(buckets[0].get_sampled_usage().get_memory_used_absolute(), 0);
        assert_eq!(buckets[0].get_sampled_usage().get_distinct_blocks(), 0);
        assert_eq!(buckets[0].get_sampled_usage().get_largest_free_block(), (0, 0, 0));
        assert_eq!(buckets[0].get_sampled_usage().get_free_blocks(), 0);
        assert_eq!(buckets[0].get_sampled_usage().get_latest_operation(), 0);
        assert_eq!(buckets[0].get_sampled_usage().get_timestamp_microseconds(), 0);
        assert_eq!(buckets[1].get_sampled_usage().get_memory_used_absolute(), 2);
        assert_eq!(buckets[1].get_sampled_usage().get_distinct_blocks(), 2);
        assert_eq!(buckets[1].get_sampled_usage().get_largest_free_block(), (3, 6, 3));
        assert_eq!(buckets[1].get_sampled_usage().get_free_blocks(), 2);
        assert_eq!(buckets[1].get_sampled_usage().get_latest_operation(), 3);
        assert_eq!(buckets[1].get_sampled_usage().get_timestamp_microseconds(), 1);
        assert_eq!(buckets[2].get_sampled_usage().get_memory_used_absolute(), 4);
        assert_eq!(buckets[2].get_sampled_usage().get_distinct_blocks(), 4);
        assert_eq!(buckets[2].get_sampled_usage().get_largest_free_block(), (5, 10, 5));
        assert_eq!(buckets[2].get_sampled_usage().get_free_blocks(), 4);
        assert_eq!(buckets[2].get_sampled_usage().get_latest_operation(), 5);
        assert_eq!(buckets[2].get_sampled_usage().get_timestamp_microseconds(), 2);
        assert_eq!(buckets[3].get_sampled_usage().get_memory_used_absolute(), 6);
        assert_eq!(buckets[3].get_sampled_usage().get_distinct_blocks(), 6);
        assert_eq!(buckets[3].get_sampled_usage().get_largest_free_block(), (6, 12, 6));
        assert_eq!(buckets[3].get_sampled_usage().get_free_blocks(), 6);
        assert_eq!(buckets[3].get_sampled_usage().get_latest_operation(), 6);
        assert_eq!(buckets[3].get_sampled_usage().get_timestamp_microseconds(), 3);
    }
    
    #[test]
    fn sample_multiple_updates_overlap_sample_interval_two() {
        let memory_usages = vec![
            // timestamp = 1
            MemoryUsage::new(1, 1, (1, 2, 1), 1, 0, 1, 1, 0),
            MemoryUsage::new(2, 2, (2, 4, 2), 2, 0, 2, 1, 0),
            MemoryUsage::new(3, 3, (3, 6, 3), 3, 0, 3, 1, 0),

            // timestamp = 2
            MemoryUsage::new(4, 4, (4, 8, 4), 4, 0, 4, 2, 0),
            MemoryUsage::new(5, 5, (5, 10, 5), 5, 0, 5, 2, 0),
            MemoryUsage::new(6, 6, (6, 12, 6), 6, 0, 6, 2, 0),

            // timestamp = 3
            MemoryUsage::new(7, 7, (7, 14, 7), 7, 0, 7, 3, 0),
            MemoryUsage::new(8, 8, (8, 16, 8), 8, 0, 8, 3, 0),
            MemoryUsage::new(9, 9, (9, 18, 9), 9, 0, 9, 3, 0),

            // timestamp = 4
            MemoryUsage::new(10, 10, (10, 20, 10), 10, 0, 10, 4, 0),
            MemoryUsage::new(11, 11, (11, 22, 11), 11, 0, 11, 4, 0),
            MemoryUsage::new(12, 12, (12, 24, 12), 12, 0, 12, 4, 0),

            // timestamp = 5
            MemoryUsage::new(13, 13, (13, 26, 13), 13, 0, 13, 5, 0),
            MemoryUsage::new(14, 14, (14, 28, 14), 14, 0, 14, 5, 0),
            MemoryUsage::new(15, 15, (15, 30, 15), 15, 0, 15, 5, 0),

            // timestamp = 6
            MemoryUsage::new(16, 16, (16, 32, 16), 16, 0, 16, 6, 0),
            MemoryUsage::new(17, 17, (17, 34, 17), 17, 0, 17, 6, 0),
            MemoryUsage::new(18, 18, (18, 36, 18), 18, 0, 18, 6, 0),
        ];

        let memory_usage_sampler = SampledMemoryUsagesFactory::new(2, memory_usages);
        let buckets = memory_usage_sampler.divide_usages_into_buckets();

        assert_eq!(buckets[0].get_first(), 1);
        assert_eq!(buckets[0].get_last(), 2);
        assert_eq!(buckets[0].get_sampled_usage().get_memory_used_absolute(), 0);
        assert_eq!(buckets[0].get_sampled_usage().get_distinct_blocks(), 0);
        assert_eq!(buckets[0].get_sampled_usage().get_largest_free_block(), (0, 0, 0));
        assert_eq!(buckets[0].get_sampled_usage().get_free_blocks(), 0);
        assert_eq!(buckets[0].get_sampled_usage().get_latest_operation(), 0);
        assert_eq!(buckets[0].get_sampled_usage().get_timestamp_microseconds(), 0);
        assert_eq!(buckets[1].get_first(), 3);
        assert_eq!(buckets[1].get_last(), 4);
        assert_eq!(buckets[1].get_sampled_usage().get_memory_used_absolute(), 3);
        assert_eq!(buckets[1].get_sampled_usage().get_distinct_blocks(), 3);
        assert_eq!(buckets[1].get_sampled_usage().get_largest_free_block(), (6, 12, 6));
        assert_eq!(buckets[1].get_sampled_usage().get_free_blocks(), 3);
        assert_eq!(buckets[1].get_sampled_usage().get_latest_operation(), 6);
        assert_eq!(buckets[1].get_sampled_usage().get_timestamp_microseconds(), 2);
        assert_eq!(buckets[2].get_first(), 4);
        assert_eq!(buckets[2].get_last(), 5);
        assert_eq!(buckets[2].get_sampled_usage().get_memory_used_absolute(), 9);
        assert_eq!(buckets[2].get_sampled_usage().get_distinct_blocks(), 9);
        assert_eq!(buckets[2].get_sampled_usage().get_largest_free_block(), (12, 24, 12));
        assert_eq!(buckets[2].get_sampled_usage().get_free_blocks(), 9);
        assert_eq!(buckets[2].get_sampled_usage().get_latest_operation(), 12);
        assert_eq!(buckets[2].get_sampled_usage().get_timestamp_microseconds(), 4);
        assert_eq!(buckets[3].get_first(), 6);
        assert_eq!(buckets[3].get_last(), 7);
        assert_eq!(buckets[3].get_sampled_usage().get_memory_used_absolute(), 15);
        assert_eq!(buckets[3].get_sampled_usage().get_distinct_blocks(), 15);
        assert_eq!(buckets[3].get_sampled_usage().get_largest_free_block(), (18, 36, 18));
        assert_eq!(buckets[3].get_sampled_usage().get_free_blocks(), 15);
        assert_eq!(buckets[3].get_sampled_usage().get_latest_operation(), 18);
        assert_eq!(buckets[3].get_sampled_usage().get_timestamp_microseconds(), 6);
    }
}


