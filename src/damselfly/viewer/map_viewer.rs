use crate::damselfly::consts::DEFAULT_MEMORYSPAN;
use crate::damselfly::memory::memory_status::MemoryStatus;
use crate::damselfly::memory::memory_update::MemoryUpdateType;
use crate::damselfly::update_interval::UpdateInterval;
use crate::damselfly::viewer::memory_canvas::MemoryCanvas;

pub struct MapViewer {
    update_intervals: Vec<UpdateInterval>,
    current_timestamp: usize,
    canvas_start: usize,
    canvas_span: usize,
    block_size: usize,
}

impl MapViewer {
    pub fn new(update_intervals: Vec<UpdateInterval>) -> MapViewer {
        let current_timestamp = update_intervals.len().saturating_sub(1);
        MapViewer {
            block_size: 1,
            update_intervals,
            current_timestamp,
            canvas_start: 0,
            canvas_span: DEFAULT_MEMORYSPAN,
        }
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

    pub fn set_block_size(&mut self, new_size: usize) {
        self.block_size = new_size;
    }

    pub fn paint_map(&mut self) -> Vec<MemoryStatus> {
        self.snap_map_to_current_update();
        let updates_till_now = self.update_intervals[0..=self.current_timestamp].to_vec();
        let mut canvas = MemoryCanvas::new(self.canvas_start, self.canvas_start + self.canvas_span, self.block_size, updates_till_now);
        canvas.render()
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
        eprintln!("start {}", self.canvas_start);
    }
}