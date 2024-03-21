use crate::damselfly::update_interval::UpdateInterval;
use crate::damselfly::update_interval::utility::Utility;
use crate::damselfly::viewer::memory_canvas::MemoryCanvas;

#[derive(Default)]
pub struct MemoryCache {
    memory_canvases: Vec<MemoryCanvas>,
    update_intervals: Vec<UpdateInterval>,
    interval: usize,
}

impl MemoryCache {
    pub fn new(block_size: usize, update_intervals: Vec<UpdateInterval>, interval: usize) -> Self {
        let (start, stop) = Utility::get_canvas_span(&update_intervals);
        let chunks = update_intervals.chunks(interval);
        let mut updates_till_now = Vec::new();
        let mut memory_canvases = Vec::new();
        for chunk in chunks {
            for update in chunk {
                updates_till_now.push(update.clone());
            }
            memory_canvases.push(MemoryCanvas::new(start, stop, block_size, updates_till_now.clone()));
        }
        Self {
            memory_canvases,
            update_intervals: updates_till_now,
            interval,
        }
    }
    
    pub fn query_cache(&self, timestamp: usize) {
        let cache_index = timestamp / self.interval;
    }
}