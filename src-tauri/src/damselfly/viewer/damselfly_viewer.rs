use std::cmp::{max, min};
use symbolic::debuginfo::pdb::pdb::FallibleIterator;
use crate::damselfly::consts::DEFAULT_CACHE_INTERVAL;
use crate::damselfly::memory::memory_parsers::MemorySysTraceParser;
use crate::damselfly::memory::memory_pool::MemoryPool;
use crate::damselfly::memory::memory_update::MemoryUpdateType;
use crate::damselfly::memory::memory_usage_factory::MemoryUsageFactory;
use crate::damselfly::memory::memory_usage_stats::MemoryUsageStats;
use crate::damselfly::viewer::damselfly_instance::DamselflyInstance;
use crate::damselfly::viewer::update_pool_factory::UpdatePoolFactory;

pub struct DamselflyViewer {
    pub damselflies: Vec<DamselflyInstance>,
    distinct_block_left_padding: usize,
    distinct_block_right_padding: usize,
}

impl DamselflyViewer {
    pub fn new(log_path: &str, binary_path: &str, cache_size: u64, distinct_block_left_padding: usize, distinct_block_right_padding: usize) -> Self {
        let mut damselfly_viewer = DamselflyViewer {
            damselflies: Vec::new(),
            distinct_block_left_padding,
            distinct_block_right_padding,
        };
        let mem_sys_trace_parser = MemorySysTraceParser::new();
        let pool_restricted_parse_results = mem_sys_trace_parser.parse_log_contents_split_by_pools(log_path, binary_path, distinct_block_left_padding, distinct_block_right_padding);
        for parse_results in &pool_restricted_parse_results {
            let (memory_updates, max_timestamp) = (parse_results.memory_updates.clone(), parse_results.max_timestamp);
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
            let memory_usage_stats = MemoryUsageFactory::new(resampled_memory_updates.clone(), distinct_block_left_padding, distinct_block_right_padding).calculate_usage_stats();
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

mod tests {
    use crate::damselfly::consts::{DEFAULT_CACHE_INTERVAL, TEST_BINARY_PATH, TEST_LOG, TEST_LOG_PATH};
    use crate::damselfly::memory::memory_parsers::MemorySysTraceParser;
    use crate::damselfly::memory::memory_usage::MemoryUsage;
    use crate::damselfly::memory::memory_usage_factory::MemoryUsageFactory;
    use crate::damselfly::viewer::damselfly_viewer::DamselflyViewer;

    fn initialise_test_log() -> DamselflyViewer {
        let mst_parser = MemorySysTraceParser::new();
        let updates = mst_parser.parse_log_directly(TEST_LOG, TEST_BINARY_PATH);
        let viewer = DamselflyViewer::new(TEST_LOG_PATH, TEST_BINARY_PATH, DEFAULT_CACHE_INTERVAL, 0, 0);
        viewer
    }

    fn initialise_log(log_path: &str) -> DamselflyViewer {
        let mst_parser = MemorySysTraceParser::new();
        let updates = mst_parser.parse_log_directly(TEST_LOG, TEST_BINARY_PATH);
        let viewer = DamselflyViewer::new(log_path, TEST_BINARY_PATH, DEFAULT_CACHE_INTERVAL, 0, 0);
        viewer
    }

    #[test]
    fn test_bug() {
        let mst_parser = MemorySysTraceParser::new();
        let mut viewer = DamselflyViewer::new("/work/dev/hp/dune/trace.log", "/work/dev/hp/dune/build/output/threadx-cortexa7-debug/ares/dragonfly-lp1/debug/defaultProductGroup/threadxApp", 1000, 0, 0);
        let map = viewer.damselflies[0].get_map_full_at_nosync_colours_truncate(0, 256);
        dbg!(&map);
    }

    #[test]
    fn test_bug_2() {
        let viewer = DamselflyViewer::new("/home/signal/Downloads/dn/trace.log", "/home/signal/Downloads/dn/threadxApp", 1000, 0, 0);
        assert_eq!(viewer.damselflies.len(), 2);
    }
}
