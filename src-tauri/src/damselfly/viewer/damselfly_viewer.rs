use std::cmp::min;
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
        let parse_results = mem_sys_trace_parser.parse_log(log_path, binary_path);
        let (memory_updates, pool_list, max_timestamp) = (parse_results.memory_updates, parse_results.pool_list, parse_results.max_timestamp);
        // cache size can't be larger than the list of updates
        let cache_size = min(cache_size, memory_updates.len() as u64);
        let memory_usage_stats = MemoryUsageFactory::new(memory_updates.clone()).calculate_usage_stats();

        let updates_sorted_into_pools = UpdatePoolFactory::sort_updates_into_pools(pool_list, memory_updates);
        for (pool, updates) in updates_sorted_into_pools {
            damselfly_viewer.spawn_damselfly(updates, memory_usage_stats.clone(), pool, max_timestamp, cache_size);
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
}
