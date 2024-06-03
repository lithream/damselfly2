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
        let (memory_cache_snapshots, updates_till_now) = MemoryCache::generate_cache(&update_intervals, interval, block_size);

        Self {
            memory_cache_snapshots,
            update_intervals: updates_till_now,
            interval,
        }
    }
    
    pub fn query_cache(&self, timestamp: usize) -> Result<Vec<MemoryStatus>, String> {
        let cache_index = (timestamp / self.interval).clamp(0, self.memory_cache_snapshots.len() - 1);
        if let Some(memory_cache_snapshot) = self.memory_cache_snapshots.get(cache_index) {
            Ok(memory_cache_snapshot.render_at(timestamp))
        } else {
            Err("[MemoryCache::query_cache]: Cache index out of bounds.".to_string())
        }
    }

    fn generate_cache(update_intervals: &Vec<UpdateInterval>, interval: usize, block_size: usize) -> (Vec<MemoryCacheSnapshot>, Vec<UpdateInterval>) {
        /*
each cache snapshot may hold a varying number of updates
for example, operation in pool 1 at t=0, operation in pool 2 at t=1, operation in pool 1 at t=2, operation in pool 2 at t=3
when caching pool 1 with an interval of 2, our chunks are [2] and the snapshot contains operations with timestamps <= 2
when caching pool 2, our chunks are [2, 3]. snapshot 1 contains operations with timestamps <= 2, so just one operation
snapshot 2 contains operations with timestamps <= 3, so the remaining operation
this is because when querying the cache, the snapshot number to use as a base is computed from the timestamp, so each snapshot
must only contain operations within [snapshot base -> snapshot base + interval]
naively splitting update_intervals into chunks of interval operations each will not achieve this
*/
        let (start, stop) = Utility::get_canvas_span(update_intervals);
        let final_timestamp = update_intervals.last().unwrap().val.get_timestamp();
        let mut update_iter = update_intervals.iter().peekable();
        
        let chunks = (interval..=final_timestamp).step_by(interval);
        let mut updates_till_now = Vec::new();
        let mut memory_cache_snapshots = Vec::new();
        let mut new_snapshot = MemoryCanvas::new(start, stop, block_size, vec![]);
        new_snapshot.insert_blocks();

        // a few logging variables
        println!("Caching operations for performance.");
        let mut chunk_no = 1;
        let chunks_len = final_timestamp / interval + 1;

        for chunk in chunks {
            // logging
            println!("Caching chunk {chunk_no} of {chunks_len}");
            chunk_no += 1;

            let mut temporary_updates = Vec::new();
            while let Some(update) = update_iter.next() {
                temporary_updates.push(update.clone());
                updates_till_now.push(update.clone());
                // break if next update should be in the next snapshot or if there are no more updates
                if let Some(next_update) = update_iter.peek() {
                    if next_update.val.get_timestamp() > chunk {
                        break;
                    }
                } else {
                    break;
                }
            }

            memory_cache_snapshots.push(MemoryCacheSnapshot::new(new_snapshot.clone(), temporary_updates.clone()));
            new_snapshot.paint_temporary_updates(temporary_updates);
        }
        /*
    let chunks = update_intervals.chunks(interval);
    let mut updates_till_now = Vec::new();
    let mut memory_cache_snapshots = Vec::new();
    let mut new_snapshot = MemoryCanvas::new(start, stop, block_size, vec![]);
    new_snapshot.insert_blocks();
    println!("Caching operations for performance.");
    let mut chunk_no = 1;
    let chunks_len = (update_intervals.len() / interval) + 1;
    for chunk in chunks {
        println!("Caching chunk {chunk_no} of {chunks_len}");
        chunk_no += 1;
        let mut temporary_updates = Vec::new();
        for update in chunk {
            temporary_updates.push(update.clone());
            updates_till_now.push(update.clone());
        }
        memory_cache_snapshots.push(MemoryCacheSnapshot::new(new_snapshot.clone(), temporary_updates.clone()));
        new_snapshot.paint_temporary_updates(temporary_updates);
    }
     */
        (memory_cache_snapshots, updates_till_now)
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