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
    pub fn new(block_size: usize, update_intervals: Vec<UpdateInterval>, interval: usize) -> Self {
        let (memory_cache_snapshots, updates_till_now) =
            MemoryCache::generate_cache(&update_intervals, interval, block_size);

        Self {
            memory_cache_snapshots,
            update_intervals: updates_till_now,
            interval,
        }
    }
    
    pub fn query_cache(&self, timestamp: usize) -> Result<Vec<MemoryStatus>, String> {
        let cache_index = (timestamp / self.interval).clamp(0, self.memory_cache_snapshots.len() - 1);
        if let Some(memory_cache_snapshot) = self.memory_cache_snapshots.get(cache_index) {
            for block in &memory_cache_snapshot.get_base().blocks {
                match block.block_status {
                    MemoryStatus::Allocated(address, _, _) => {
                        if address == 3789773696 {
                            eprintln!("break");
                        }
                    }
                    MemoryStatus::PartiallyAllocated(address, _, _) => {
                        if address == 3789773696 {
                            eprintln!("break");
                        }
                    }
                    MemoryStatus::Free(address, _, _) => {
                        if address == 3789773696 {
                            eprintln!("break");
                        }
                    }
                    MemoryStatus::Unused => {}
                }
            }
            Ok(memory_cache_snapshot.render_at(timestamp))
        } else {
            Err("[MemoryCache::query_cache]: Cache index out of bounds.".to_string())
        }
    }

    fn generate_cache(update_intervals: &Vec<UpdateInterval>, interval: usize, block_size: usize) -> (Vec<MemoryCacheSnapshot>, Vec<UpdateInterval>) {
        let (start, stop) = Utility::get_canvas_span(update_intervals);
        let final_timestamp = update_intervals.last().unwrap().val.get_timestamp();

        let mut buckets: HashMap<usize, Vec<UpdateInterval>> = HashMap::new();
        
        // Categories update into buckets in the hashmap
        for update in update_intervals {
            let cache_index = update.val.get_timestamp() / interval;
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

    pub fn change_block_size(&mut self, new_block_size: usize) {
        eprintln!("[MemoryCache::change_block_size]: Recomputing cache. Changing block size to: {new_block_size}");
        self.memory_cache_snapshots = Self::generate_cache(&self.update_intervals, self.interval, new_block_size).0;
    }
}

mod tests {
    use crate::damselfly::consts::DEFAULT_CACHE_INTERVAL;
    use crate::damselfly::memory::memory_cache::MemoryCache;
    use crate::damselfly::memory::memory_parsers::MemorySysTraceParser;
    use crate::damselfly::update_interval::update_interval_factory::UpdateIntervalFactory;

    #[test]
    fn benchmark() {
        let mst_parser = MemorySysTraceParser::new();
        let memory_updates = mst_parser.parse_log("./trace4.log", "./threadxApp").memory_updates;
        let update_interval_factory = UpdateIntervalFactory::new(memory_updates);
        let update_intervals = update_interval_factory.construct_enum_vector();
        let memory_cache = MemoryCache::new(4, update_intervals, DEFAULT_CACHE_INTERVAL as usize);
    }
}