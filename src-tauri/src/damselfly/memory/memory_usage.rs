use std::cmp::Ordering;

#[derive(Debug, Default, Clone)]
pub struct MemoryUsage {
    memory_used_absolute: i128,
    distinct_blocks: u128,
    // (start, end, size)
    largest_free_block: (usize, usize, usize),
    free_blocks: usize,
    latest_operation: usize,
    timestamp_microseconds: u64,
    timestamp: u64
}

impl MemoryUsage {
    pub fn new(memory_used_absolute: i128, distinct_blocks: u128, largest_free_block: (usize, usize, usize), free_blocks: usize, latest_operation: usize, timestamp_microseconds: u64, timestamp: u64) -> MemoryUsage {
        MemoryUsage {
            memory_used_absolute,
            distinct_blocks,
            largest_free_block,
            free_blocks,
            latest_operation,
            timestamp_microseconds,
            timestamp
        }
    }
}

impl MemoryUsage {
    pub fn get_memory_used_absolute(&self) -> i128 {
        self.memory_used_absolute
    }
    pub fn set_memory_used_absolute(&mut self, memory_used_absolute: i128) {
        self.memory_used_absolute = memory_used_absolute;
    }
    
    pub fn get_distinct_blocks(&self) -> u128 {
        self.distinct_blocks
    }
    
    pub fn set_distinct_blocks(&mut self, distinct_blocks: u128) {
        self.distinct_blocks = distinct_blocks;
    }
    
    pub fn get_latest_operation(&self) -> usize {
        self.latest_operation
    }
    
    pub fn set_latest_operation(&mut self, latest_operation: usize) {
        self.latest_operation = latest_operation;
    }

    pub fn get_largest_free_block(&self) -> (usize, usize, usize) { self.largest_free_block }
    
    pub fn set_largest_free_block(&mut self, largest_free_block: (usize, usize, usize)) {
        self.largest_free_block = largest_free_block;
    }

    pub fn get_free_blocks(&self) -> usize { self.free_blocks }
    
    pub fn get_timestamp(&self) -> u64 { self.timestamp }
    
    pub fn set_timestamp(&mut self, timestamp: u64) {
        self.timestamp = timestamp
    }
    
    pub fn set_free_blocks(&mut self, free_blocks: usize) {
        self.free_blocks = free_blocks;
    }
    
    pub fn get_timestamp_microseconds(&self) -> u64 { self.timestamp_microseconds }
    
    pub fn set_timestamp_microseconds(&mut self, timestamp_microseconds: u64) {
        self.timestamp_microseconds = timestamp_microseconds;
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

mod tests {
    use crate::damselfly::memory::memory_usage::MemoryUsage;

    #[test]
    fn ordering_test() {
        let base = MemoryUsage::new(128, 4, (0, 0, 0), 0, 4, 0, 0);
        let larger = MemoryUsage::new(256, 3, (0, 0, 0), 0, 3, 0, 0);
        let equal = MemoryUsage::new(128, 32, (0, 0, 0), 0, 32, 0, 0);
        assert_eq!(base, equal);
        assert!(base < larger);
        assert!(equal < larger);
        assert_ne!(larger, equal);
    }
}