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
        for chunk in chunks {
            let mut temporary_updates = Vec::new();
            let new_snapshot = MemoryCanvas::new(start, stop, block_size, updates_till_now.clone());
            for update in chunk {
                temporary_updates.push(update.clone());
                updates_till_now.push(update.clone());
            }
            memory_cache_snapshots.push(MemoryCacheSnapshot::new(new_snapshot, temporary_updates));
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
}