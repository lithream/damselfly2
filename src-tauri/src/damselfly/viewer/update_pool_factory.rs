use crate::damselfly::memory::memory_pool::MemoryPool;
use crate::damselfly::memory::memory_pool_list::MemoryPoolList;
use crate::damselfly::memory::memory_update::MemoryUpdateType;

pub struct UpdatePoolFactory {
}

impl UpdatePoolFactory {
    pub fn sort_updates_into_pools(pools: MemoryPoolList, update_intervals: Vec<MemoryUpdateType>) -> Vec<(MemoryPool, Vec<MemoryUpdateType>)> {
        let mut pools_with_updates = Vec::new();
        for pool in pools.get_pools() {
            pools_with_updates.push((pool.clone(), Vec::new()));
        }

        // Sort pools as get_pools returns a HashSet which may not be deterministic
        pools_with_updates.sort_by(
            |(prev_pool, _prev_updates), (next_pool, _next_updates)|
            prev_pool.get_start().cmp(&next_pool.get_start())
        );
        
        for update in update_intervals {
            // Find the matching pool and add to its corresponding list of updates
            for (pool, updates_in_pool) in pools_with_updates.iter_mut() {
                if pool.contains(update.get_start(), update.get_end()) {
                    updates_in_pool.push(update.clone());
                }
            }
        };
        
        pools_with_updates
    }
}