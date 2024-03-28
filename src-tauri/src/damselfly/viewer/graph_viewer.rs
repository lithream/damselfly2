use crate::damselfly::memory::memory_usage::MemoryUsage;

enum GraphMode {
    NORMAL,
    MARKED
}

pub struct GraphViewer {
    memory_usage_snapshots: Vec<MemoryUsage>,
    current_highlight: Option<usize>,
    saved_highlight: usize,
    max_usage: i128,
    max_distinct_blocks: usize,
    left_mark: usize,
    right_mark: usize,
    graph_mode: GraphMode
}

impl GraphViewer {
    pub fn new(memory_usage_snapshots: Vec<MemoryUsage>, max_usage: i128, max_distinct_blocks: usize) -> GraphViewer {
        GraphViewer {
            memory_usage_snapshots,
            current_highlight: None,
            saved_highlight: 0,
            max_usage,
            max_distinct_blocks,
            left_mark: 0,
            right_mark: 0,
            graph_mode: GraphMode::NORMAL,
        }
    }

    pub fn get_usage_plot_points(&self) -> Vec<[f64; 2]> {
        let mut vector = Vec::new();
        for (index, snapshot) in self.memory_usage_snapshots.iter().enumerate() {
            let memory_used_percentage =
                (snapshot.get_memory_used_absolute() as f64 * 100.0) / self.get_max_usage() as f64;
            vector.push([index as f64, memory_used_percentage]);
        }
        vector
    }
    
    pub fn get_usage_plot_points_realtime_sampled(&self, sample_interval: u64) -> Vec<[f64; 2]> {
        Vec::new()
    }

    pub fn get_distinct_blocks_plot_points(&self) -> Vec<[f64; 2]> {
        let mut vector = Vec::new();
        for (index, snapshot) in self.memory_usage_snapshots.iter().enumerate() {
            let distinct_blocks_percentage =
                (snapshot.get_distinct_blocks() as f64 * 100.0) / self.get_max_distinct_blocks() as f64;
            vector.push([index as f64, distinct_blocks_percentage]);
        }
        vector
    }
    
    pub fn get_largest_free_block_plot_points(&self) -> Vec<[f64; 2]> {
        let mut vector = Vec::new();
        for (index, usage) in self.memory_usage_snapshots.iter().enumerate() {
            vector.push([index as f64, usage.get_largest_free_block().2 as f64]);
        }
        vector
    }
    
    pub fn get_free_blocks_plot_points(&self) -> Vec<[f64; 2]> {
        let mut vector = Vec::new();
        for (index, usage) in self.memory_usage_snapshots.iter().enumerate() {
            vector.push([index as f64, usage.get_free_blocks() as f64]);
        }
        vector
    }

    pub fn get_total_operations(&self) -> usize {
        self.memory_usage_snapshots.len()
    }

    pub fn get_highlight(&self) -> usize {
        if let Some(highlight) = self.current_highlight {
            return highlight;
        }
        self.saved_highlight
    }

    pub fn set_current_highlight(&mut self, new_highlight: usize) {
        self.current_highlight = Some(new_highlight);
    }

    pub fn set_saved_highlight(&mut self, new_highlight: usize) {
        self.saved_highlight = new_highlight;
    }

    pub fn clear_current_highlight(&mut self) {
        self.current_highlight = None;
    }

    pub fn get_saved_highlight(&self) -> usize {
        self.saved_highlight
    }

    fn get_max_usage(&self) -> i128 {
        self.max_usage
    }

    fn get_max_distinct_blocks(&self) -> usize {
        self.max_distinct_blocks
    }
}
