use crate::damselfly::memory::memory_parsers::MemorySysTraceParser;
use crate::damselfly::memory::memory_status::MemoryStatus;
use crate::damselfly::memory::memory_update::MemoryUpdateType;
use crate::damselfly::memory::memory_usage_factory::MemoryUsageFactory;
use crate::damselfly::update_interval::update_interval_factory::UpdateIntervalFactory;
use crate::damselfly::viewer::graph_viewer::GraphViewer;
use crate::damselfly::viewer::map_viewer::MapViewer;

pub struct DamselflyViewer {
    graph_viewer: GraphViewer,
    map_viewer: MapViewer,
}

impl DamselflyViewer {
    pub fn new(log_path: &str, binary_path: &str) -> DamselflyViewer {
        let mem_sys_trace_parser = MemorySysTraceParser::new();
        let memory_updates = mem_sys_trace_parser.parse_log(log_path, binary_path);
        let (memory_usages, max_usage) = MemoryUsageFactory::new(memory_updates.clone()).calculate_usage_stats();
        let graph_viewer = GraphViewer::new(memory_usages, max_usage);
        let update_intervals = UpdateIntervalFactory::new(memory_updates).construct_enum_vector();
        let map_viewer = MapViewer::new(update_intervals);
        DamselflyViewer {
            graph_viewer,
            map_viewer
        }
    }

    pub fn get_map(&mut self) -> Vec<MemoryStatus> {
        self.sync_viewers();
        self.map_viewer.paint_map()
    }

    pub fn get_graph(&self) -> Vec<[f64; 2]> {
        self.graph_viewer.get_plot_points()
    }

    pub fn get_total_operations(&self) -> usize {
        self.graph_viewer.get_total_operations()
    }

    pub fn get_current_operation(&self) -> MemoryUpdateType {
        self.map_viewer.get_current_operation()
    }

    pub fn set_graph_current_highlight(&mut self, new_highlight: usize) {
        self.graph_viewer.set_current_highlight(new_highlight);
    }

    pub fn set_graph_saved_highlight(&mut self, new_highlight: usize) {
        self.graph_viewer.set_saved_highlight(new_highlight);
    }

    pub fn clear_graph_current_highlight(&mut self) {
        self.graph_viewer.clear_current_highlight();
    }

    pub fn get_graph_highlight(&self) -> usize {
        self.graph_viewer.get_highlight()
    }

    pub fn set_block_size(&mut self, new_size: usize) {
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