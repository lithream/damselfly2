pub struct MemoryUsageRealtimeFactory {
    memory_updates: Vec<MemoryUpdateType>,
    lowest_address: usize,
    highest_address: usize,
    lapper: Lapper<usize, MemoryUpdateType>,
}

impl MemoryUsageFactory {
    pub fn new(memory_updates: Vec<MemoryUpdateType>) -> MemoryUsageFactory {
        MemoryUsageFactory {
            memory_updates,
            lowest_address: usize::MAX,
            highest_address: usize::MIN,
            lapper: Lapper::new(vec![]),
        }
    }

    pub fn load_memory_updates(&mut self, updates: Vec<MemoryUpdateType>) {
        self.memory_updates = updates;
    }

    pub fn calculate_usage_stats(&mut self) -> (Vec<MemoryUsage>, i128, usize) {
        let mut current_usage = 0;
        let mut max_usage = 0;
        let mut memory_usages = Vec::new();

        let mut distinct_block_counter = DistinctBlockCounter::default();
        let mut max_distinct_blocks = 0;
        let current_timestamp = self.memory_updates.first().

        for (index, update) in self.memory_updates.iter().enumerate() {

        }
    }
}