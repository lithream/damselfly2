use crate::damselfly2::memory::memory_usage::MemoryUsage;
use crate::damselfly2::memory::memory_usage_factory::MemoryUsageFactory;

enum GraphMode {
    NORMAL,
    MARKED
}

pub struct GraphViewer {
    memory_usage_snapshots: Vec<MemoryUsage>,
    current_highlight: usize,
    saved_highlight: usize,
    max_usage: i128,
    left_mark: usize,
    right_mark: usize,
    graph_mode: GraphMode
}

impl GraphViewer {
    pub fn new(memory_usage_snapshots: Vec<MemoryUsage>, max_usage: i128) -> GraphViewer {
        let mut viewer = GraphViewer {
            memory_usage_snapshots,
            current_highlight: 0,
            saved_highlight: 0,
            max_usage,
            left_mark: 0,
            right_mark: 0,
            graph_mode: GraphMode::NORMAL,
        };
        viewer
    }

    pub fn get_plot_points(&self) -> Vec<[f64; 2]> {
        let mut vector = Vec::new();
        for (index, snapshot) in self.memory_usage_snapshots.iter().enumerate() {
            let memory_used_percentage = 
                (snapshot.get_memory_used_absolute() as f64 * 100.0) / self.get_max_usage() as f64;
            vector.push([index as f64, memory_used_percentage]);
        }
        Vec::new()
    }

    pub fn get_current_highlight(&self) -> usize {
        self.current_highlight
    }

    pub fn get_saved_highlight(&self) -> usize {
        self.saved_highlight
    }
    fn get_max_usage(&self) -> i128 {
        self.max_usage
    }
}
