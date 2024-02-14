use std::cmp::Ordering;

#[derive(Debug, Default, Clone)]
pub struct MemoryUsage {
    pub memory_used_absolute: usize,
    pub blocks: usize,
    pub latest_operation: usize,
}

impl MemoryUsage {
    pub fn new(memory_used_absolute: usize, blocks: usize, latest_operation: usize) -> MemoryUsage {
        MemoryUsage {
            memory_used_absolute,
            blocks,
            latest_operation,
        }
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