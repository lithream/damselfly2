use std::cmp::min;
use crate::damselfly::memory::memory_parsers::MemorySysTraceParser;
use crate::damselfly::memory::memory_pool::MemoryPool;
use crate::damselfly::memory::memory_update::MemoryUpdateType;
use crate::damselfly::memory::memory_usage_factory::MemoryUsageFactory;
use crate::damselfly::memory::memory_usage_stats::MemoryUsageStats;
use crate::damselfly::viewer::damselfly_instance::DamselflyInstance;

pub struct DamselflyViewer {
    pub damselflies: Vec<DamselflyInstance>,
}

impl DamselflyViewer {
    pub fn new(log_path: &str, binary_path: &str, cache_size: u64, distinct_block_left_padding: usize, distinct_block_right_padding: usize) -> Self {
        let mut damselfly_viewer = DamselflyViewer {
            damselflies: Vec::new(),
        };
        let mem_sys_trace_parser = MemorySysTraceParser::new();
        let pool_restricted_parse_results = mem_sys_trace_parser.parse_log_contents_split_by_pools(log_path, binary_path, distinct_block_left_padding, distinct_block_right_padding);
        for parse_results in &pool_restricted_parse_results {
            let (memory_updates, max_timestamp) = (parse_results.memory_updates.clone(), parse_results.max_timestamp);
            let (pool_start, pool_stop) = (parse_results.pool.get_start(), parse_results.pool.get_start() + parse_results.pool.get_size());
            let mut resampled_memory_updates = Vec::new();
            // This should really be iter_mut, but I don't want to break anything
            for (index, memory_update) in memory_updates.iter().enumerate() {
                let mut resampled_memory_update = memory_update.clone();
                resampled_memory_update.set_timestamp(index);
                resampled_memory_updates.push(resampled_memory_update);
            }

            // Compensate for padding
            for memory_update in resampled_memory_updates.iter_mut() {
                memory_update.set_absolute_address(memory_update.get_absolute_address() - distinct_block_left_padding);
                memory_update.set_absolute_size(memory_update.get_absolute_size() + distinct_block_right_padding);
            }
            
            let cache_size = min(cache_size, resampled_memory_updates.len() as u64);
            let memory_usage_stats = MemoryUsageFactory::new(resampled_memory_updates.clone(), 
                                                             distinct_block_left_padding,
                                                             distinct_block_right_padding,
                                                             pool_start,
                                                             pool_stop,
                                                            ).calculate_usage_stats();
            damselfly_viewer.spawn_damselfly(resampled_memory_updates, memory_usage_stats, parse_results.pool.clone(), max_timestamp, cache_size);
        }

        damselfly_viewer
    }

    fn spawn_damselfly(&mut self, memory_updates: Vec<MemoryUpdateType>, memory_usage_stats: MemoryUsageStats, pool: MemoryPool, max_timestamp: u64, cache_size: u64) {
        self.damselflies.push(
            DamselflyInstance::new(
                pool.get_name().to_string(),
                memory_updates,
                memory_usage_stats,
                pool.get_start(),
                pool.get_start() + pool.get_size(),
                cache_size as usize,
                max_timestamp,
            )
        );
    }
}
