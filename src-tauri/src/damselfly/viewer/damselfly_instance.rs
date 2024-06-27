//! A single instance of Damselfly, which contains a graph and a map for a single pool.
//! To have multiple pools, instantiate a DamselflyInstance for each pool and store them in
//! DamselflyViewer.
use crate::damselfly::memory::memory_usage_stats::MemoryUsageStats;
use rust_lapper::Lapper;
use crate::damselfly::consts::{DEFAULT_OPERATION_LOG_SIZE, DEFAULT_SAMPLE_INTERVAL};
use crate::damselfly::memory::memory_status::MemoryStatus;
use crate::damselfly::memory::memory_update::MemoryUpdateType;
use crate::damselfly::memory::sampled_memory_usages::SampledMemoryUsages;
use crate::damselfly::update_interval::update_interval_factory::UpdateIntervalFactory;
use crate::damselfly::viewer::graph_viewer::GraphViewer;
use crate::damselfly::viewer::map_viewer::MapViewer;

pub struct DamselflyInstance {
    name: String,
    graph_viewer: GraphViewer,
    map_viewer: MapViewer,
    full_lapper: Lapper<usize, MemoryUpdateType>,
}

impl DamselflyInstance {
    /// Constructor.
    ///
    /// # Arguments
    ///
    /// * `name`: Name.
    /// * `memory_updates`: Updates to store in this DamselflyInstance. Create these using a MemoryParser.
    /// * `memory_usage_stats`: Stats to plot on the graph.
    /// * `lowest_address`: Lowest address - from pool bounds computed during parsing.
    /// * `highest_address`: Highest address - from pool bounds computed during parsing.
    /// * `cache_size`: Interval at which maps should be cached.
    /// * `max_timestamp`: Max absolute operation timestamp to show on the graph - computed during parsing.
    ///
    /// returns: DamselflyInstance
    pub fn new(name: String, memory_updates: Vec<MemoryUpdateType>, memory_usage_stats: MemoryUsageStats,
               lowest_address: usize, highest_address: usize, cache_size: usize, max_timestamp: u64,
    ) -> Self {
        let memory_usages = memory_usage_stats.get_memory_usages();
        let max_usage = memory_usage_stats.get_max_usage();
        let max_distinct_blocks = memory_usage_stats.get_max_distinct_blocks();
        let max_free_blocks = memory_usage_stats.get_max_free_blocks();
        let max_free_segment_fragmentation = memory_usage_stats.get_max_free_segment_fragmentation();
        let max_largest_free_block = memory_usage_stats.get_max_largest_free_block();

        let sampled_memory_usages =
            SampledMemoryUsages::new(DEFAULT_SAMPLE_INTERVAL, memory_usages.clone());

        let graph_viewer = GraphViewer::new(
            memory_usages.clone(),
            sampled_memory_usages,
            max_usage,
            max_free_blocks,
            max_distinct_blocks as usize,
            max_free_segment_fragmentation,
            max_largest_free_block,
            max_timestamp,
        );

        let update_intervals = UpdateIntervalFactory::new(memory_updates).construct_enum_vector();
        let map_viewer = MapViewer::new(name.clone(), update_intervals.clone(), lowest_address, highest_address, cache_size as u64);
        let full_lapper = Lapper::new(update_intervals);

        Self {
            name,
            graph_viewer,
            map_viewer,
            full_lapper,
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Renders the memory map in full at a specified timestamp, truncating regions that are too large
    /// for legibility.
    ///
    /// # Arguments
    ///
    /// * `timestamp`: Timestamp to render the map at.
    /// * `truncate_after`: How large a region must be before it gets truncated.
    ///
    /// returns: (timestamp, Vec<(parent_address, status, address)>)
    ///
    /// Timestamp is the timestamp of the map. The Vec stores a list of tuples, where each tuple
    /// represents a single block on the map (which could span several bytes).
    /// Each block has:
    /// a parent_address (address of the most recent allocation/free that overlaps this block)
    /// a status (representing whether it is allocated, partially allocated, freed or unused)
    /// an address (the block's own address)
    pub fn get_map_full_at_nosync_colours_truncate(
        &mut self,
        timestamp: u64,
        truncate_after: u64,
    ) -> (u64, Vec<(i64, u64, usize)>) {
        self.map_viewer.set_timestamp(timestamp as usize);
        let full_map = self.map_viewer.paint_map_full_from_cache();

        // parent address, address, status
        let mut result: Vec<(i64, u64, usize)> = Vec::new();
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
                MemoryStatus::Allocated(_, _, _, _) => 3,
                MemoryStatus::PartiallyAllocated(_, _, _, _) => 2,
                MemoryStatus::Free(_, _, _, _) => 1,
                MemoryStatus::Unused(_) => 0,
            };

            let parent_address: i64 = if block.get_parent_address().is_none() {
                -1
            } else {
                block.get_parent_address().unwrap() as i64
            };

            let address = block.get_address();
            result.push((parent_address, status, address));
        }

        (timestamp, result)
    }


    /// Renders the memory map in full at a specified timestamp, truncating regions that are too large
    /// for legibility.
    ///
    /// # Arguments
    ///
    /// * `timestamp`: A realtime timestamp that will be translated into an absolute operation timestamp.
    /// * `truncate_after`: How large a region must be before it gets truncated.
    ///
    /// returns: (timestamp, Vec<(parent_address, status, address)>)
    ///
    /// Timestamp is the timestamp of the map. The Vec stores a list of tuples, where each tuple
    /// represents a single block on the map (which could span several bytes).
    /// Each block has:
    /// a parent_address (address of the most recent allocation/free that overlaps this block)
    /// a status (representing whether it is allocated, partially allocated, freed or unused)
    /// an address (the block's own address)
    pub fn get_map_full_at_nosync_colours_truncate_realtime_sampled(
        &mut self,
        timestamp: u64,
        truncate_after: u64,
    ) -> (u64, Vec<(i64, u64, usize)>) {
        let operation_timestamp = self
            .graph_viewer
            .get_operation_timestamp_of_realtime_timestamp(timestamp);
        eprintln!("[DamselflyInstance::get_map_full_at_nosync_colours_truncate_realtime_sampled]: timestamp: {timestamp}");
        eprintln!("[DamselflyInstance::get_map_full_at_nosync_colours_truncate_realtime_sampled]: operation timestamp: {operation_timestamp}");
        self.get_map_full_at_nosync_colours_truncate(operation_timestamp, truncate_after)
    }

    /// Gets a graph, but with filler values so that all pools have the same number of
    /// points.
    ///
    /// returns: Vec<[timestamp, y-value]>
    pub fn get_usage_graph(&self) -> Vec<[f64; 2]> {
        self.graph_viewer.get_usage_plot_points()
    }

    /// Gets a graph, but without filler values, so different pools may have different numbers
    /// of points.
    ///
    /// returns: Vec<[timestamp, y-value]>
    pub fn get_usage_graph_no_fallbacks(&self) -> Vec<[f64;2]> {
        self.graph_viewer.get_usage_plot_points_no_fallbacks()
    }

    /// Gets a graph in realtime.
    ///
    /// returns: Vec<[timestamp, y-value]>
    pub fn get_usage_graph_realtime_sampled(&self) -> Vec<[f64; 2]> {
        self.graph_viewer.get_usage_plot_points_realtime_sampled()
    }

    /// Gets a graph, but with filler values so that all pools have the same number of
    /// points.
    ///
    /// returns: Vec<[timestamp, y-value]>
    pub fn get_distinct_blocks_graph(&self) -> Vec<[f64; 2]> {
        self.graph_viewer.get_distinct_blocks_plot_points()
    }

    /// Gets a graph, but without filler values, so different pools may have different numbers
    /// of points.
    ///
    /// returns: Vec<[timestamp, y-value]>
    pub fn get_distinct_blocks_graph_no_fallbacks(&self) -> Vec<[f64; 2]> {
        self.graph_viewer.get_distinct_blocks_plot_points_no_fallbacks()
    }

    /// Gets a graph in realtime.
    ///
    /// returns: Vec<[timestamp, y-value]>
    pub fn get_distinct_blocks_graph_realtime_sampled(&self) -> Vec<[f64; 2]> {
        self.graph_viewer.get_distinct_blocks_plot_points_realtime_sampled()
    }

    /// Gets a graph, but without filler values, so different pools may have different numbers
    /// of points.
    ///
    /// returns: Vec<[timestamp, y-value]>
    pub fn get_free_segment_fragmentation_graph_no_fallbacks(&self) -> Vec<[f64; 2]> {
        self.graph_viewer.get_free_segment_fragmentation_plot_points_no_fallbacks()
    }

    /// Gets a graph in realtime.
    ///
    /// returns: Vec<[timestamp, y-value]>
    pub fn get_free_segment_fragmentation_graph_realtime_sampled(&self) -> Vec<[f64; 2]> {
        self.graph_viewer.get_free_segment_fragmentation_plot_points_realtime_sampled()
    }

    /// Gets a graph, but without filler values, so different pools may have different numbers
    /// of points.
    ///
    /// returns: Vec<[timestamp, y-value]>
    pub fn get_largest_free_block_graph_no_fallbacks(&self) -> Vec<[f64; 2]> {
        self.graph_viewer.get_largest_free_block_plot_points_no_fallbacks()
    }

    /// Gets a graph in realtime.
    ///
    /// returns: Vec<[timestamp, y-value]>
    pub fn get_largest_free_block_graph_realtime_sampled(&self) -> Vec<[f64; 2]> {
        self.graph_viewer.get_largest_free_block_plot_points_realtime_sampled()
    }

    /// Gets a graph, but with filler values so that all pools have the same number of
    /// points.
    ///
    /// returns: Vec<[timestamp, y-value]>
    pub fn get_largest_block_graph(&self) -> Vec<[f64; 2]> {
        self.graph_viewer.get_largest_free_block_plot_points()
    }

    /// Gets a graph, but without filler values, so different pools may have different numbers
    /// of points.
    ///
    /// returns: Vec<[timestamp, y-value]>
    pub fn get_largest_block_graph_no_fallbacks(&self) -> Vec<[f64; 2]> {
        self.graph_viewer.get_largest_free_block_plot_points_no_fallbacks()
    }

    /// Gets a graph in realtime.
    ///
    /// returns: Vec<[timestamp, y-value]>
    pub fn get_largest_block_graph_realtime_sampled(&self) -> Vec<[f64; 2]> {
        self.graph_viewer
            .get_largest_free_block_plot_points_realtime_sampled()
    }

    /// Gets a graph, but with filler values so that all pools have the same number of
    /// points.
    ///
    /// returns: Vec<[timestamp, y-value]>
    pub fn get_free_blocks_graph(&self) -> Vec<[f64; 2]> {
        self.graph_viewer.get_free_blocks_plot_points()
    }

    /// Gets a graph, but without filler values, so different pools may have different numbers
    /// of points.
    ///
    /// returns: Vec<[timestamp, y-value]>
    pub fn get_free_blocks_graph_no_fallbacks(&self) -> Vec<[f64; 2]> {
        self.graph_viewer.get_free_blocks_plot_points_no_fallbacks()
    }

    /// Gets a graph in realtime.
    ///
    /// returns: Vec<[timestamp, y-value]>
    pub fn get_free_blocks_graph_realtime_sampled(&self) -> Vec<[f64; 2]> {
        self.graph_viewer
            .get_free_blocks_plot_points_realtime_sampled()
    }

    /// Gets the latest operation shown in the current map state.
    pub fn get_current_operation(&self) -> MemoryUpdateType {
        self.map_viewer.get_current_operation()
    }

    /// Gets the full operation history.
    pub fn get_operation_history(&self) -> Vec<MemoryUpdateType> {
        self.map_viewer
            .get_update_history(DEFAULT_OPERATION_LOG_SIZE)
    }

    /// Queries a block to get all updates that overlap it from t=0 until the specified timestamp.
    ///
    /// # Arguments
    ///
    /// * `address`: Address of the block (absolute).
    /// * `timestamp`: Timestamp to query until.
    ///
    /// returns: Vec<MemoryUpdateType, Global>
    pub fn query_block(&self, address: usize, timestamp: usize) -> Vec<MemoryUpdateType> {
        eprintln!("[DamselflyInstance::query_block]: optimestamp: {timestamp}");
        eprintln!("[DamselflyInstance::query_block]: address: {address}");
        self.full_lapper
            .find(address, address + self.map_viewer.get_block_size())
            .filter(|interval| interval.val.get_timestamp() <= timestamp)
            .map(|interval| interval.val.clone())
            .collect()
    }

    /// Queries a block to get all updates that overlap it.
    ///
    /// # Arguments
    ///
    /// * `address`: Address of the block.
    ///
    /// returns: Vec<MemoryUpdateType, Global>
    pub fn query_block_naive(&self, address: usize) -> Vec<MemoryUpdateType> {
        eprintln!("[DamselflyInstance::query_block_naive]: address: {address}");
        self.full_lapper
            .find(address, address + self.map_viewer.get_block_size())
            .map(|interval| interval.val.clone())
            .collect()
    }

    /// Queries a block to get all updates that overlap it from t=0 until the specified realtime timestamp.
    ///
    /// # Arguments
    ///
    /// * `address`: Address of the block.
    /// * `timestamp`: Realtime timestamp.
    ///
    /// returns: Vec<MemoryUpdateType, Global>
    pub fn query_block_realtime(&self, address: usize, timestamp: usize) -> Vec<MemoryUpdateType> {
        let timestamp = self.graph_viewer.get_operation_timestamp_of_realtime_timestamp(timestamp as u64) as usize;
        eprintln!("[DamselflyInstance::query_block_realtime]: realtime converted to optimestamp: {timestamp}");
        self.full_lapper
            .find(address, address + self.map_viewer.get_block_size())
            .filter(|interval| interval.val.get_timestamp() <= timestamp)
            .map(|interval| interval.val.clone())
            .collect()
    }

    pub fn set_map_block_size(&mut self, new_size: usize) {
        self.map_viewer.set_block_size(new_size);
    }
}