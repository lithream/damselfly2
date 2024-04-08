use crate::damselfly::memory::memory_usage::MemoryUsage;

pub struct MemoryUsageStats {
    memory_usages: Vec<MemoryUsage>,
    max_usage: i128,
    max_free_blocks: u128,
    max_distinct_blocks: u128,
}

impl MemoryUsageStats {
    pub fn new(memory_usages: Vec<MemoryUsage>, max_usage: i128, max_free_blocks: u128, max_distinct_blocks: u128) -> Self {
        Self {
            memory_usages,
            max_usage,
            max_free_blocks,
            max_distinct_blocks,
        }
    }
    
    pub fn get_memory_usages(&self) -> &Vec<MemoryUsage> {
        &self.memory_usages
    }
    
    pub fn get_max_usage(&self) -> i128 {
        self.max_usage
    }
    
    pub fn get_max_free_blocks(&self) -> u128 {
        self.max_free_blocks
    }
    
    pub fn get_max_distinct_blocks(&self) -> u128 {
        self.max_distinct_blocks
    }
}