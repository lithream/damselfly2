use egui::TextBuffer;
use crate::damselfly2::memory::memory_update::MemoryUpdateType;
use crate::damselfly2::memory::
use crate::damselfly2::memory::memory_parsers::MemorySysTraceParser;
use crate::damselfly2::memory::memory_usage_factory::MemoryUsageFactory;
use crate::damselfly2::update_interval::update_interval_factory::UpdateIntervalFactory;
use crate::damselfly2::viewer::graph_viewer::GraphViewer;

pub struct DamselflyViewer {

}

impl DamselflyViewer {
    pub fn new(log_path: &str, binary_path: &str) -> DamselflyViewer {
        let mem_sys_trace_parser = MemorySysTraceParser::new();
        let memory_updates = mem_sys_trace_parser.parse_log(log_path, binary_path);
        let (memory_usages, max_usage) = MemoryUsageFactory::new(memory_updates.clone()).calculate_usage_stats();
        let update_intervals = UpdateIntervalFactory::new(memory_updates).construct_enum_vector();
        let graph_viewer = GraphViewer::new(memory_usages, max_usage);
        let
    }
}