//! MemoryCache
//! 
//! Stores memory cache snapshots and provides methods to query and modify the cache.
//! 
//! Do not use MemoryCacheSnapshot directly - it is best to generate and manage the cache
//! using a MemoryCache object.
use std::collections::HashMap;
use crate::damselfly::memory::memory_cache_snapshot::MemoryCacheSnapshot;
use crate::damselfly::memory::memory_status::MemoryStatus;
use crate::damselfly::update_interval::UpdateInterval;
use crate::damselfly::update_interval::utility::Utility;
use crate::damselfly::viewer::memory_canvas::MemoryCanvas;

#[derive(Default)]
pub struct MemoryCache {
    memory_cache_snapshots: Vec<MemoryCacheSnapshot>,
    update_intervals: Vec<UpdateInterval>,
    interval: usize,
}

impl MemoryCache {
    /// Constructor
    /// 
    /// # Arguments 
    /// 
    /// * `block_size`: Bytes spanned by each block of the map.
    /// * `update_intervals`: Vector of all updates
    /// * `interval`: Interval between each cache. e.g. 1000 implies a cache is generated at 
    /// t=0, t=1000 and so on.
    /// 
    /// returns: MemoryCache 
    pub fn new(block_size: usize, update_intervals: Vec<UpdateInterval>, interval: usize) -> Self {
        let (memory_cache_snapshots, updates_till_now) =
            MemoryCache::generate_cache(&update_intervals, interval, block_size);

        Self {
            memory_cache_snapshots,
            update_intervals: updates_till_now,
            interval,
        }
    }
    
    /// Renders the map at a specific timestamp using stored caches.
    /// 
    /// # Arguments 
    /// 
    /// * `timestamp`: Timestamp in operation time to fetch the map for.
    /// 
    /// returns: Result<Vec<MemoryStatus, Global>, String> 
    pub fn query_cache(&self, timestamp: usize) -> Result<Vec<MemoryStatus>, String> {
        let cache_index = (timestamp / self.interval).clamp(0, self.memory_cache_snapshots.len() - 1);
        if let Some(memory_cache_snapshot) = self.memory_cache_snapshots.get(cache_index) {
            let offset = timestamp - (cache_index * self.interval);
            Ok(memory_cache_snapshot.render_this_many(offset))
        } else {
            Err("[MemoryCache::query_cache]: Cache index out of bounds.".to_string())
        }
    }

    /// Generates the cache by separating updates into buckets of size interval and painting a map
    /// for each one.
    /// Not exposed for public use; use MemoryCache::new() instead, which calls this internally.
    /// 
    /// # Arguments 
    /// 
    /// * `update_intervals`: Vec of updates.
    /// * `interval`: Interval between each cached map.
    /// * `block_size`: Bytes spanned by each block in the map.
    /// 
    /// returns: (Vec<MemoryCacheSnapshot, Global>, Vec<Interval<usize, MemoryUpdateType>, Global>) 
    fn generate_cache(update_intervals: &Vec<UpdateInterval>, interval: usize, block_size: usize) -> (Vec<MemoryCacheSnapshot>, Vec<UpdateInterval>) {
        let (start, stop) = Utility::get_canvas_span(update_intervals);
        let final_timestamp = update_intervals.len() - 1;

        let mut buckets: HashMap<usize, Vec<UpdateInterval>> = HashMap::new();
        
        // Categories update into buckets in the hashmap
        for (index, update) in update_intervals.iter().enumerate() {
            let cache_index = index / interval;
            buckets
                .entry(cache_index)
                .and_modify(|bucket| bucket.push(update.clone()))
                .or_insert(vec![update.clone()]);
        }
        
        // Iterate through every possible cache index from [0..=final_timestamp / interval]
        let mut memory_cache_snapshots = Vec::new();
        let mut current_canvas = MemoryCanvas::new(start, stop, block_size, vec![]);
        current_canvas.insert_blocks();
        
        for cache_index in 0..=final_timestamp / interval {
            let updates_in_bucket = buckets.get(&cache_index).cloned().unwrap_or(Vec::new());
            memory_cache_snapshots.push(MemoryCacheSnapshot::new(current_canvas.clone(), updates_in_bucket.clone()));
            current_canvas.paint_temporary_updates(updates_in_bucket.clone());
        }

        (memory_cache_snapshots, update_intervals.clone())
    }

    /// Changes the block size. This regenerates the entire cache which is quite slow, so use this
    /// sparingly.
    /// 
    /// # Arguments 
    /// 
    /// * `new_block_size`: New block size to change to.
    /// 
    /// returns: () 
    pub fn change_block_size(&mut self, new_block_size: usize) {
        eprintln!("[MemoryCache::change_block_size]: Recomputing cache. Changing block size to: {new_block_size}");
        self.memory_cache_snapshots = Self::generate_cache(&self.update_intervals, self.interval, new_block_size).0;
    }
}
