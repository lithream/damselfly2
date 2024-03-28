use std::collections::HashMap;
use std::slice::Iter;
use crate::damselfly::memory::memory_usage::MemoryUsage;
use crate::damselfly::memory::utility::Utility;

pub struct MemoryUsageSampler {
    current_time: u64,
    sample_interval: u64,
    memory_usages: Vec<MemoryUsage>,
}

impl MemoryUsageSampler {
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
    fn divide_usages_into_buckets(&self) -> Vec<MemoryUsage> {
        let mut buckets = HashMap::new();
        for usage in self.memory_usages {
            let rounded_timestamp = Utility::round_to_nearest_multiple_of(usage.get_timestamp_microseconds(), self.sample_interval);
            buckets.entry(rounded_timestamp).or_insert(Vec::new()).push(usage);
        }

        let mut averaged_buckets = Vec::new();
        let mut bucket_keys: Vec<u64> = buckets.keys().cloned().collect();
        bucket_keys.sort();
        let mut previous_averaged_usage = MemoryUsage::new(0, 0, (0, 0, 0), 0, 0, 0);
        for key in 0..buckets.len() {
            let bucket_index = (key as u64) * self.sample_interval;

            match buckets.get(&bucket_index) {
                None => {
                    averaged_buckets.push(previous_averaged_usage.clone());
                }
                Some(usages) => {
                    let mut total_usage = usages.iter().reduce(|mut acc, usage| {
                        acc.set_memory_used_absolute(acc.get_memory_used_absolute() + usage.get_memory_used_absolute());
                        acc.set_distinct_blocks(acc.get_distinct_blocks() + usage.get_distinct_blocks());
                        acc.set_free_blocks(acc.get_free_blocks() + usage.get_free_blocks());
                        acc
                    }).unwrap();
                    total_usage.set_memory_used_absolute(total_usage.get_memory_used_absolute() / usages.len() as i128);
                    total_usage.set_distinct_blocks(total_usage.get_distinct_blocks() / usages.len());
                    total_usage.set_free_blocks(total_usage.get_free_blocks() / usages.len());
                    averaged_buckets.push(total_usage.clone());
                }
            }
        }
        averaged_buckets
    }

}