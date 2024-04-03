use crate::damselfly::memory::memory_usage::MemoryUsage;

#[derive(Clone)]
pub struct MemoryUsageSample {
    memory_usages: Vec<MemoryUsage>,
    first: u64,
    last: u64,
    sampled_usage: MemoryUsage
}

impl MemoryUsageSample {
    pub fn new(memory_usages: Vec<MemoryUsage>, first: u64, last: u64, sampled_usage: MemoryUsage) -> Self {
        Self {
            memory_usages,
            first,
            last,
            sampled_usage,
        }
    }
    
    pub fn get_memory_usages(&self) -> &Vec<MemoryUsage> {
        &self.memory_usages
    }
    
    pub fn get_first(&self) -> u64 {
        self.first
    }
    
    pub fn get_last(&self) -> u64 {
        self.last
    }
    
    pub fn get_sampled_usage(&self) -> MemoryUsage {
        self.sampled_usage.clone()
    }
}