//! A MemoryUsage created by sampling a group of MemoryUsages. Used for rendering a realtime graph,
//! where each point on the x-axis represents real time such as 100ms and thus represents the
//! average of all MemoryUsage points contained within that span.
//! 
//! This is just a wrapper struct and does not do any computation to find the average of MemoryUsages.
use crate::damselfly::memory::memory_usage::MemoryUsage;

#[derive(Clone, Debug)]
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