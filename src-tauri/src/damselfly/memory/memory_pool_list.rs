use std::collections::HashSet;
use crate::damselfly::memory::memory_pool::MemoryPool;

#[derive(Default)]
pub struct MemoryPoolList {
    pools: HashSet<MemoryPool>
}

impl MemoryPoolList {
    pub fn new(pools: HashSet<MemoryPool>) -> Self {
        Self {
            pools
        }
    }
    
    pub fn add_pool(&mut self, pool: MemoryPool) {
        self.pools.insert(pool);
    }

    pub fn get_pools(&self) -> &HashSet<MemoryPool> {
        &self.pools
    }
}