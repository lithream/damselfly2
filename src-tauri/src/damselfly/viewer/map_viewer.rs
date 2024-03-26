use crate::damselfly::consts::{DEFAULT_BLOCK_SIZE, DEFAULT_MEMORYSPAN};
use crate::damselfly::memory::memory_cache::MemoryCache;
use crate::damselfly::memory::memory_status::MemoryStatus;
use crate::damselfly::memory::memory_update::{MemoryUpdate, MemoryUpdateType};
use crate::damselfly::update_interval::UpdateInterval;
use crate::damselfly::viewer::memory_canvas::MemoryCanvas;

pub struct MapViewer {
    cache: MemoryCache,
    update_intervals: Vec<UpdateInterval>,
    current_timestamp: usize,
    canvas_start: usize,
    canvas_span: usize,
    block_size: usize,
    lowest_address: usize,
    highest_address: usize,
}

impl MapViewer {
    pub fn new(update_intervals: Vec<UpdateInterval>) -> MapViewer {
        let current_timestamp = update_intervals.len().saturating_sub(1);

        let lowest_address = update_intervals.iter().min_by(|prev, next| {
            prev.val.get_absolute_address().cmp(&next.val.get_absolute_address())
        }).expect("[MapViewer::new]: Cannot find lowest address").val.get_absolute_address();

        let highest_address = update_intervals.iter().max_by(|prev, next| {
            prev.val.get_absolute_address().cmp(&next.val.get_absolute_address())
        }).expect("[MapViewer::new]: Cannot find highest address").val.get_absolute_address();

        MapViewer {
            cache: MemoryCache::new(DEFAULT_BLOCK_SIZE, update_intervals.clone(), 1000),
            update_intervals,
            current_timestamp,
            canvas_start: 0,
            canvas_span: DEFAULT_MEMORYSPAN,
            block_size: DEFAULT_BLOCK_SIZE,
            lowest_address,
            highest_address,
        }
    }

    pub fn get_update_history(&self, history_size: usize) -> Vec<MemoryUpdateType> {
        self.update_intervals
            .iter()
            .take(self.current_timestamp)
            .rev()
            .take(history_size)
            .map(|update_interval| update_interval.val.clone())
            .collect()
    }

    pub fn get_updates_from(&self, start: usize, end: usize) -> Vec<UpdateInterval> {
        self.update_intervals[start..=end].to_vec()
    }

    pub fn get_update_intervals(&self) -> &Vec<UpdateInterval> {
        &self.update_intervals
    }

    pub fn get_lowest_address(&self) -> usize {
        self.lowest_address
    }

    pub fn get_highest_address(&self) -> usize {
        self.highest_address
    }

    pub fn set_timestamp(&mut self, new_timestamp: usize) {
        self.current_timestamp = new_timestamp.clamp(usize::MIN, self.update_intervals.len() - 1);
    }

    pub fn pan_forward(&mut self, units: usize) {
        self.canvas_start += units;
    }

    pub fn pan_backward(&mut self, units: usize) {
        self.canvas_start = self.canvas_start.saturating_sub(units);
    }

    pub fn set_map_span(&mut self, new_span: usize) {
        self.canvas_span = new_span;
    }

    pub fn get_block_size(&self) -> usize {
        self.block_size
    }
    
    pub fn set_block_size(&mut self, new_size: usize) {
        let span_scale_factor = new_size as f64 / self.block_size as f64;
        self.set_map_span((self.canvas_span as f64 * span_scale_factor).round() as usize);
        self.block_size = new_size;
    }

    pub fn snap_and_paint_map(&mut self) -> Vec<MemoryStatus> {
        self.snap_map_to_current_update();
        self.paint_map()
    }
    
    pub fn paint_map(&mut self) -> Vec<MemoryStatus> {
        let updates_till_now = self.update_intervals[0..=self.current_timestamp].to_vec();
        let mut canvas = MemoryCanvas::new(self.canvas_start, self.canvas_start + self.canvas_span, self.block_size, updates_till_now);
        canvas.render()
    }

    pub fn paint_map_full(&self) -> Vec<MemoryStatus> {
        let updates_till_now = self.update_intervals[0..=self.current_timestamp].to_vec();
        let mut canvas = MemoryCanvas::new(self.lowest_address, self.highest_address, self.block_size, updates_till_now);
        canvas.render()
    }
    
    pub fn paint_map_full_from_cache(&self) -> Vec<MemoryStatus> {
        self.cache.query_cache(self.current_timestamp).unwrap()
    }

    pub fn get_current_operation(&self) -> MemoryUpdateType {
        self.update_intervals.get(self.current_timestamp)
            .expect("[MapViewer::get_current_operation]: Current timestamp not found in update intervals Vec")
            .val
            .clone()
    }

    fn snap_map_to_current_update(&mut self) {
        let current_update = self.update_intervals.get(self.current_timestamp)
            .expect("[MapViewer::snap_map_to_current_update]: Current timestamp out of bounds of update intervals");
        self.canvas_start = current_update.start.saturating_sub(self.canvas_span / 2);
    }
}