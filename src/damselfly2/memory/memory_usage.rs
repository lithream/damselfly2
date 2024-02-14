use std::cmp::Ordering;

#[derive(Debug, Default, Clone)]
pub struct MemoryUsage {
    memory_used_absolute: i128,
    distinct_blocks: usize,
    latest_operation: usize,
}

impl MemoryUsage {
    pub fn new(memory_used_absolute: i128, distinct_blocks: usize, latest_operation: usize) -> MemoryUsage {
        MemoryUsage {
            memory_used_absolute,
            distinct_blocks,
            latest_operation,
        }
    }
}

impl MemoryUsage {
    pub fn get_memory_used_absolute(&self) -> i128 {
        self.memory_used_absolute
    }
    
    pub fn get_distinct_blocks(&self) -> usize {
        self.distinct_blocks
    }
    
    pub fn get_latest_operation(&self) -> usize {
        self.latest_operation
    }
}

impl Eq for MemoryUsage {}

impl PartialEq<Self> for MemoryUsage {
    fn eq(&self, other: &Self) -> bool {
        self.memory_used_absolute == other.memory_used_absolute
    }
}

impl PartialOrd<Self> for MemoryUsage {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.memory_used_absolute.partial_cmp(&other.memory_used_absolute)
    }
}

impl Ord for MemoryUsage {
    fn cmp(&self, other: &Self) -> Ordering {
        self.memory_used_absolute.cmp(&other.memory_used_absolute)
    }
}