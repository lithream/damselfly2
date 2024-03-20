use crate::damselfly::update_interval::UpdateInterval;
use crate::damselfly::update_interval::utility::Utility;
use crate::damselfly::viewer::memory_canvas::MemoryCanvas;

#[derive(Default)]
pub struct MemoryCache {
    memory_canvases: Vec<MemoryCanvas>,
    update_intervals: Vec<UpdateInterval>,
}

impl MemoryCache {
    pub fn new(block_size: usize, update_intervals: Vec<UpdateInterval>, interval: usize) -> Self {
        let (start, stop) = Utility::get_canvas_span(&update_intervals);
        let chunks = update_intervals.chunks(interval);
        let mut updates_till_now = Vec::new();
        let mut cache = MemoryCache::default();
        for chunk in chunks {
            for update in chunk {
                updates_till_now.push(update.clone());
            }
            cache.memory_canvases.push(MemoryCanvas::new(start, stop, block_size, updates_till_now.clone()));
        }
        cache
    }

}