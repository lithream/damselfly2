use crate::damselfly2::memory::memory_usage::MemoryUsage;

enum GraphMode {
    NORMAL,
    MARKED
}

pub struct GraphViewer {
    memory_usage_snapshots: Vec<MemoryUsage>,
    current_highlight: usize,
    saved_highlight: usize,
    max_usage: usize,
    left_mark: usize,
    right_mark: usize,
    graph_mode: GraphMode
}

impl GraphViewer {
    pub fn new(snapshots: Vec<MemoryUsage>) -> GraphViewer {
        let mut viewer = GraphViewer {
            memory_usage_snapshots: snapshots,
            current_highlight: 0,
            saved_highlight: 0,
            max_usage: 0,
            left_mark: 0,
            right_mark: 0,
            graph_mode: GraphMode::NORMAL,
        };
        viewer.calculate_max_usage();
        viewer
    }

    pub fn get_plot_points(&self) -> Vec<[f64; 2]> {
        let mut vector = Vec::new();
        for snapshot in self.memory_usage_snapshots {
            let memory_used_percentage = snapshot.memory_used_absolute as f64 * 100.0 /
        }
        Vec::new()
    }

    fn calculate_max_usage(&mut self) {
        let max_usage = 0;
        for snapshot in self.memory_usage_snapshots {
            
        }
    }
}