use std::cmp::Ordering;

#[derive(Debug, Default, Clone)]
pub struct MemoryUsage {
    memory_used_absolute: i128,
    distinct_blocks: usize,
    largest_free_block: (usize, usize, usize),
    free_blocks: usize,
    latest_operation: usize,
}

impl MemoryUsage {
    pub fn new(memory_used_absolute: i128, distinct_blocks: usize, largest_free_block: (usize, usize, usize), free_blocks: usize, latest_operation: usize) -> MemoryUsage {
        MemoryUsage {
            memory_used_absolute,
            distinct_blocks,
            largest_free_block,
            free_blocks,
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

    pub fn get_largest_free_block(&self) -> (usize, usize, usize) { self.largest_free_block }

    pub fn get_free_blocks(&self) -> usize { self.free_blocks }
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

mod tests {
    use crate::damselfly::memory::memory_usage::MemoryUsage;

    #[test]
    fn ordering_test() {
        let base = MemoryUsage::new(128, 4, 0, 4);
        let larger = MemoryUsage::new(256, 3, 0, 3);
        let equal = MemoryUsage::new(128, 32, 0, 32);
        assert_eq!(base, equal);
        assert!(base < larger);
        assert!(equal < larger);
        assert_ne!(larger, equal);
    }
}