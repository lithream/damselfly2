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
}

impl DamselflyViewer {
    pub fn new(log_path: &str, binary_path: &str, cache_size: u64) -> Self {
        let mut damselfly_viewer = DamselflyViewer {
            damselflies: Vec::new()
        };
        let mem_sys_trace_parser = MemorySysTraceParser::new();
        let pool_restricted_parse_results = mem_sys_trace_parser.parse_log_contents_split_by_pools(log_path, binary_path);
//        let parse_results = mem_sys_trace_parser.parse_log(log_path, binary_path);
        for parse_results in &pool_restricted_parse_results {
            let (memory_updates, max_timestamp) = (parse_results.memory_updates.clone(), parse_results.max_timestamp);
            let mut resampled_memory_updates = Vec::new();
            for (index, memory_update) in memory_updates.iter().enumerate() {
                let mut resampled_memory_update = memory_update.clone();
                resampled_memory_update.set_timestamp(index);
                resampled_memory_updates.push(resampled_memory_update);
            }
            let cache_size = min(cache_size, resampled_memory_updates.len() as u64);
            let memory_usage_stats = MemoryUsageFactory::new(resampled_memory_updates.clone()).calculate_usage_stats();
            damselfly_viewer.spawn_damselfly(resampled_memory_updates, memory_usage_stats, parse_results.pool.clone(), max_timestamp, cache_size);
        }
        /*
        let (memory_updates, pool_list, max_timestamp) = (parse_results.memory_updates, parse_results.pool_list, parse_results.max_timestamp);
        // cache size can't be larger than the list of updates
        let cache_size = min(cache_size, memory_updates.len() as u64);

        let updates_sorted_into_pools = UpdatePoolFactory::sort_updates_into_pools(pool_list, memory_updates);
        for (pool, updates) in updates_sorted_into_pools {
            let memory_usage_stats = MemoryUsageFactory::new(updates.clone()).calculate_usage_stats();
            damselfly_viewer.spawn_damselfly(updates, memory_usage_stats, pool, max_timestamp, cache_size);
        }

         */
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
        let viewer = DamselflyViewer::new(TEST_LOG_PATH, TEST_BINARY_PATH, DEFAULT_CACHE_INTERVAL);
        viewer
    }

    fn initialise_log(log_path: &str) -> DamselflyViewer {
        let mst_parser = MemorySysTraceParser::new();
        let updates = mst_parser.parse_log_directly(TEST_LOG, TEST_BINARY_PATH);
        let viewer = DamselflyViewer::new(log_path, TEST_BINARY_PATH, DEFAULT_CACHE_INTERVAL);
        viewer
    }

    #[test]
    fn test_bug() {
        let mst_parser = MemorySysTraceParser::new();
        let updates = mst_parser.parse_log("/home/signal/dev/trace.log", "/home/signal/dev/threadxApp");
        let viewer = DamselflyViewer::new("/home/signal/dev/trace.log", "/home/signal/dev/threadxApp", 1000);
    }

    #[test]
    fn test_bug_2() {
        let viewer = DamselflyViewer::new("/home/signal/Downloads/dn/trace.log", "/home/signal/Downloads/dn/threadxApp", 1000);
        assert_eq!(viewer.damselflies.len(), 2);
    }
}
