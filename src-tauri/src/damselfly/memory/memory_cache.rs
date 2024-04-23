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
        let (start, stop) = Utility::get_canvas_span(&update_intervals);
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
        Self {
            memory_cache_snapshots,
            update_intervals: updates_till_now,
            interval,
        }
    }
    
    pub fn query_cache(&self, timestamp: usize) -> Result<Vec<MemoryStatus>, String> {
        let cache_index = timestamp / self.interval;
        let cache_offset = timestamp - (cache_index * self.interval);
        if let Some(memory_cache_snapshot) = self.memory_cache_snapshots.get(cache_index) {
            Ok(memory_cache_snapshot.render_at(cache_offset))
        } else {
            Err("[MemoryCache::query_cache]: Cache index out of bounds.".to_string())
        }
    }

    pub fn change_block_size(&mut self, new_block_size: usize) {
        eprintln!("[MemoryCache::change_block_size]: Recomputing cache. Changing block size to: {new_block_size}");
        let (start, stop) = Utility::get_canvas_span(&self.update_intervals);
        let chunks = self.update_intervals.chunks(self.interval);
        let mut updates_till_now = Vec::new();
        let mut memory_cache_snapshots = Vec::new();
        let mut new_snapshot = MemoryCanvas::new(start, stop, new_block_size, vec![]);
        new_snapshot.insert_blocks();
        for chunk in chunks {
            let mut temporary_updates = Vec::new();
            for update in chunk {
                temporary_updates.push(update.clone());
                updates_till_now.push(update.clone());
            }
            memory_cache_snapshots.push(MemoryCacheSnapshot::new(new_snapshot.clone(), temporary_updates.clone()));
            new_snapshot.paint_temporary_updates(temporary_updates);
        }
        self.memory_cache_snapshots = memory_cache_snapshots;
    }
}

mod tests {
    use crate::damselfly::memory::memory_cache::MemoryCache;
    use crate::damselfly::memory::memory_parsers::MemorySysTraceParser;
    use crate::damselfly::update_interval::update_interval_factory::UpdateIntervalFactory;

    #[test]
    fn benchmark() {
        let mst_parser = MemorySysTraceParser::new();
        let memory_updates = mst_parser.parse_log("./trace4.log", "./threadxApp").memory_updates;
        let update_interval_factory = UpdateIntervalFactory::new(memory_updates);
        let update_intervals = update_interval_factory.construct_enum_vector();
        let memory_cache = MemoryCache::new(4, update_intervals, 100);
    }
}