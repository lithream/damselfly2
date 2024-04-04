use std::cmp::max;
use std::time::Instant;
use rust_lapper::Lapper;
use crate::damselfly::consts::{DEFAULT_BLOCKS_TO_TRUNCATE, DEFAULT_OPERATION_LOG_SIZE, DEFAULT_SAMPLE_INTERVAL};
use crate::damselfly::memory::memory_parsers::MemorySysTraceParser;
use crate::damselfly::memory::memory_status::MemoryStatus;
use crate::damselfly::memory::memory_update::MemoryUpdateType;
use crate::damselfly::memory::memory_usage_factory::MemoryUsageFactory;
use crate::damselfly::memory::sampled_memory_usages::SampledMemoryUsages;
use crate::damselfly::update_interval::update_interval_factory::UpdateIntervalFactory;
use crate::damselfly::update_interval::update_queue_compressor::UpdateQueueCompressor;
use crate::damselfly::update_interval::UpdateInterval;
use crate::damselfly::viewer::graph_viewer::GraphViewer;
use crate::damselfly::viewer::map_viewer::MapViewer;
use crate::damselfly::memory::sampled_memory_usages_factory::SampledMemoryUsagesFactory;

pub struct DamselflyViewer {
    graph_viewer: GraphViewer,
    map_viewer: MapViewer,
}

impl DamselflyViewer {
    pub fn new(log_path: &str, binary_path: &str) -> DamselflyViewer {
        let mem_sys_trace_parser = MemorySysTraceParser::new();
        let memory_updates = mem_sys_trace_parser.parse_log(log_path, binary_path);
        let (memory_usages, max_usage, max_distinct_blocks) =
            MemoryUsageFactory::new(memory_updates.clone()).calculate_usage_stats();
        eprintln!("memory_usages len: {}", memory_usages.len());
        let sampled_memory_usages = SampledMemoryUsages::new(DEFAULT_SAMPLE_INTERVAL, memory_usages.clone());
        eprintln!("sampled len: {}", sampled_memory_usages.get_samples().len());
        let graph_viewer = GraphViewer::new(memory_usages, sampled_memory_usages, max_usage, max_distinct_blocks);
        let update_intervals = UpdateIntervalFactory::new(memory_updates).construct_enum_vector();
        let map_viewer = MapViewer::new(update_intervals);
        DamselflyViewer {
            graph_viewer,
            map_viewer
        }
    }

    pub fn get_map(&mut self) -> Vec<MemoryStatus> {
        self.sync_viewers();
        self.map_viewer.snap_and_paint_map()
    }

    pub fn get_map_full(&mut self) -> Vec<MemoryStatus> {
        self.sync_viewers();
        self.map_viewer.paint_map_full()
    }

    pub fn get_map_full_nosync(&self) -> Vec<MemoryStatus> {
        self.map_viewer.paint_map_full()
    }

    pub fn get_map_full_at(&mut self, timestamp: usize) -> Vec<MemoryStatus> {
        eprintln!("get map full at {timestamp}");
        self.set_graph_saved_highlight(timestamp);
        self.map_viewer.paint_map_full()
    }

    pub fn get_map_full_at_nosync(&mut self, timestamp: usize) -> Vec<MemoryStatus> {
        self.map_viewer.set_timestamp(timestamp);
        self.map_viewer.paint_map_full()
    }

    pub fn get_map_full_at_nosync_colours_truncate(&mut self, timestamp: u64, truncate_after: u64) -> (u64, Vec<(i64, u64)>) {
        let start = Instant::now();
        self.map_viewer.set_timestamp(timestamp as usize);
        let full_map = self.map_viewer.paint_map_full_from_cache();
        let stop = start.elapsed();
        eprintln!("get map full at nosync colours truncate: paint map full: {}", stop.as_micros());
        eprintln!("full map size: {}", full_map.len());

        let mut result: Vec<(i64, u64)> = Vec::new();
        let mut consecutive_identical_blocks = 0;

        for (index, block) in full_map.iter().enumerate() {
            if let Some(prev_block) = full_map.get(index.saturating_sub(1)) {
                if prev_block == block {
                    consecutive_identical_blocks += 1;
                } else {
                    consecutive_identical_blocks = 0;
                }
            }

            if consecutive_identical_blocks > truncate_after {
                continue;
            }

            let status = match block {
                MemoryStatus::Allocated(_, _, _) => 3,
                MemoryStatus::PartiallyAllocated(_, _, _) => 2,
                MemoryStatus::Free(_, _, _) => 1,
                MemoryStatus::Unused => 0,
            };

            let parent_address: i64 = if block.get_parent_address().is_none() {
                -1
            } else {
                block.get_parent_address().unwrap() as i64
            };
            result.push((parent_address, status));
        }

        (timestamp, result)
    }

    pub fn get_map_full_at_nosync_colours_truncate_realtime_sampled(&mut self, timestamp: u64, truncate_after: u64) -> (u64, Vec<(i64, u64)>) {
        let operation_timestamp = self.graph_viewer.get_operation_timestamp_of_realtime_timestamp(timestamp);
        self.get_map_full_at_nosync_colours_truncate(operation_timestamp, truncate_after)
    }

    pub fn get_usage_graph(&self) -> Vec<[f64; 2]> {
        self.graph_viewer.get_usage_plot_points()
    }
    
    pub fn get_usage_graph_realtime_sampled(&self) -> Vec<[f64; 2]> {
        self.graph_viewer.get_usage_plot_points_realtime_sampled()
    }

    pub fn get_distinct_blocks_graph(&self) -> Vec<[f64; 2]> {
        self.graph_viewer.get_distinct_blocks_plot_points()
    }

    pub fn get_distinct_blocks_graph_realtime_sampled(&self) -> Vec<[f64; 2]> {
        self.graph_viewer.get_distinct_blocks_plot_points_realtime_sampled()
    }
    
    pub fn get_largest_block_graph(&self) -> Vec<[f64; 2]> {
        self.graph_viewer.get_largest_free_block_plot_points()
    }

    pub fn get_largest_block_graph_realtime_sampled(&self) -> Vec<[f64; 2]> {
        self.graph_viewer.get_largest_free_block_plot_points_realtime_sampled()
    }

    pub fn get_free_blocks_graph(&self) -> Vec<[f64; 2]> {
        self.graph_viewer.get_free_blocks_plot_points()
    }

    pub fn get_free_blocks_graph_realtime_sampled(&self) -> Vec<[f64; 2]> {
        self.graph_viewer.get_free_blocks_plot_points_realtime_sampled()
    }

    pub fn get_free_blocks_stats(&self) -> (usize, usize) {
        let updates_till_now = self.map_viewer.get_updates_from(0, self.get_graph_highlight());
        let updates_till_now: Vec<MemoryUpdateType> = updates_till_now.iter()
            .map(|update| update.val.clone())
            .collect();
        let compressed_allocs = UpdateQueueCompressor::compress_to_allocs(&updates_till_now);
        let compressed_intervals = UpdateIntervalFactory::new(compressed_allocs).construct_enum_vector();
        let mut lapper = Lapper::new(compressed_intervals);
        lapper.merge_overlaps();

        let mut largest_free_block_size: usize = 0;
        let mut lapper_iter = lapper.iter().peekable();

        while let Some(current_block) = lapper_iter.next() {
            if let Some(next_block) = lapper_iter.peek() {
                let current_free_block_size = next_block.val.get_start() - current_block.val.get_start();
                largest_free_block_size = max(largest_free_block_size, current_free_block_size);
            }
        }

        let mut largest_free_block_size = 0;
        let mut free_blocks = 0;
        let mut left = self.map_viewer.get_lowest_address();
        let mut right = left + 1;
        let highest_address = self.map_viewer.get_highest_address();

        while right < highest_address {
            while lapper.find(left, right).count() == 0{
                right += 1;
            }
            largest_free_block_size = max(largest_free_block_size, right - left);
            free_blocks += 1;
            left = right;
            right = left + 1;
        }
        (largest_free_block_size, free_blocks)
    }

    pub fn get_total_operations(&self) -> usize {
        self.graph_viewer.get_total_operations()
    }

    pub fn get_current_operation(&self) -> MemoryUpdateType {
        dbg!(&self.map_viewer.get_current_operation());
        self.map_viewer.get_current_operation()
    }

    pub fn get_operation_history(&self) -> Vec<MemoryUpdateType> {
        self.map_viewer.get_update_history(DEFAULT_OPERATION_LOG_SIZE)
    }

    pub fn get_graph_highlight(&self) -> usize {
        self.graph_viewer.get_highlight()
    }

    pub fn get_all_intervals(&self) -> &Vec<UpdateInterval> {
        self.map_viewer.get_update_intervals()
    }

    pub fn set_graph_current_highlight(&mut self, new_highlight: usize) {
        self.graph_viewer.set_current_highlight(new_highlight);
    }

    pub fn set_graph_saved_highlight(&mut self, new_highlight: usize) {
        self.graph_viewer.set_saved_highlight(new_highlight);
    }

    pub fn get_block_size(&self) -> usize {
        self.map_viewer.get_block_size()
    }

    pub fn clear_graph_current_highlight(&mut self) {
        self.graph_viewer.clear_current_highlight();
    }

    pub fn set_map_block_size(&mut self, new_size: usize) {
        self.map_viewer.set_block_size(new_size);
    }

    pub fn set_map_span(&mut self, new_span: usize) {
        self.map_viewer.set_map_span(new_span);
    }

    pub fn sync_viewers(&mut self) {
        let current_timestamp = self.graph_viewer.get_highlight();
        self.map_viewer.set_timestamp(current_timestamp);
    }
}

mod tests {
    use crate::damselfly::consts::{TEST_BINARY_PATH, TEST_LOG, TEST_LOG_PATH};
    use crate::damselfly::memory::memory_parsers::MemorySysTraceParser;
    use crate::damselfly::memory::memory_usage::MemoryUsage;
    use crate::damselfly::memory::memory_usage_factory::MemoryUsageFactory;
    use crate::damselfly::viewer::damselfly_viewer::DamselflyViewer;

    fn initialise_test_log() -> DamselflyViewer {
        let mst_parser = MemorySysTraceParser::new();
        let updates = mst_parser.parse_log_directly(TEST_LOG, TEST_BINARY_PATH);
        let viewer = DamselflyViewer::new(TEST_LOG_PATH, TEST_BINARY_PATH);
        viewer
    }
    
    fn initialise_log(log_path: &str) -> DamselflyViewer {
        let mst_parser = MemorySysTraceParser::new();
        let updates = mst_parser.parse_log_directly(TEST_LOG, TEST_BINARY_PATH);
        let viewer = DamselflyViewer::new(log_path, TEST_BINARY_PATH);
        viewer
    }
    
    #[test]
    fn benchmark() {
        let mut viewer = DamselflyViewer::new("/home/oracle/dev/damselfly2/src-tauri/trace3.log", "/home/oracle/dev/damselfly2/src-tauri/threadxApp");
//        let mut viewer = initialise_log("./tracequarter.log");
        viewer.get_map_full_at_nosync_colours_truncate(13000, 256);
    }
}