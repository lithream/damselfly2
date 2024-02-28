pub struct MemoryUsageRealtime {
    memory_used_absolute: i128,
    distinct_blocks: usize,
    latest_operations: (u64, u64)
}

impl MemoryUsageRealtime {
    pub fn new(memory_used_absolute: i128, distinct_blocks: usize, latest_operations: (u64, u64)) -> MemoryUsageRealtime {
        MemoryUsageRealtime {
            memory_used_absolute,
            distinct_blocks,
            latest_operations,
        }
    }

    pub fn get_memory_used_absolute(&self) -> i128 {
        self.memory_used_absolute
    }

    pub fn get_distinct_blocks(&self) -> usize {
        self.distinct_blocks
    }

    pub fn get_latest_operation(&self) -> (usize, usize) {
        self.latest_operations
    }


}

impl Eq for MemoryUsageRealtime {}

impl PartialEq<Self> for MemoryUsageRealtime {
    fn eq(&self, other: &Self) -> bool {
        self.memory_used_absolute == other.memory_used_absolute
    }

    impl PartialOrd<Self> for MemoryUsageRealtime {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            self.memory_used_absolute.partial_cmp(&other.memory_used_absolute)
        }
    }
}

impl Ord for MemoryUsageRealtime {
    fn cmp(&self, other: &Self) -> Ordering {
        self.memory_used_absolute.cmp(&other.memory_Used_absolute)
    }
}