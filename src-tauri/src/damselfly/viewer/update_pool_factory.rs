use crate::damselfly::memory::memory_pool::MemoryPool;
use crate::damselfly::memory::memory_pool_list::MemoryPoolList;
use crate::damselfly::memory::memory_update::MemoryUpdateType;
use crate::damselfly::update_interval::UpdateInterval;

pub struct UpdatePoolFactory {
}

impl UpdatePoolFactory {
    pub fn sort_updates_into_pools(pools: MemoryPoolList, update_intervals: Vec<MemoryUpdateType>) -> Vec<(MemoryPool, Vec<MemoryUpdateType>)> {
        let mut pools_with_updates = Vec::new();
        for pool in pools.get_pools() {
            pools_with_updates.push((pool.clone(), Vec::new()));
        }
        
        for update in update_intervals {
            // Find the matching pool and its corresponding list of updates
            if let Some((_matching_pool, updates_in_pool)) = pools_with_updates.iter_mut().find(
                |(pool, _updates)| {
                    let pool_start = pool.get_start();
                    let pool_end = pool_start + pool.get_size();
                    pool_start < update.get_start() && pool_end > update.get_end()
                }) {
                // Push update into the pool's corresponding list of updates
                updates_in_pool.push(update);
            }
        };
        
        pools_with_updates
    }
}