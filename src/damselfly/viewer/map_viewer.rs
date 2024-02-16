use crate::damselfly::consts::DEFAULT_MEMORYSPAN;
use crate::damselfly::memory::memory_status::MemoryStatus;
use crate::damselfly::memory::memory_update::MemoryUpdateType;
use crate::damselfly::update_interval::UpdateInterval;
use crate::damselfly::viewer::memory_canvas::MemoryCanvas;

pub struct MapViewer {
    update_intervals: Vec<UpdateInterval>,
    current_highlight: usize,
    current_timestamp: usize,
    canvas_start: usize,
    canvas_stop: usize,
}

impl MapViewer {
    pub fn new(update_intervals: Vec<UpdateInterval>) -> MapViewer {
        let current_timestamp = update_intervals.len().saturating_sub(1);
        MapViewer {
            update_intervals,
            current_highlight: 0,
            current_timestamp,
            canvas_start: 0,
            canvas_stop: DEFAULT_MEMORYSPAN,
        }
    }

    pub fn set_timestamp(&mut self, new_timestamp: usize) {
        self.current_timestamp = new_timestamp.clamp(usize::MIN, self.update_intervals.len() - 1);
    }

    pub fn pan_forward(&mut self, units: usize) {
        self.canvas_start += units;
        self.canvas_stop += units;
    }

    pub fn pan_backward(&mut self, units: usize) {
        self.canvas_start = self.canvas_start.saturating_sub(units);
        self.canvas_stop = self.canvas_stop.saturating_sub(units);
    }

    pub fn paint_map(&self) -> Vec<MemoryStatus> {
        let updates_till_now = self.update_intervals[0..=self.current_timestamp].to_vec();
        let canvas = MemoryCanvas::new(self.canvas_start, self.canvas_stop, updates_till_now);
        canvas.render()
    }

    pub fn get_current_operation(&self) -> MemoryUpdateType {
        self.update_intervals.get(self.current_timestamp)
            .expect("[MapViewer::get_current_operation]: Current timestamp not found in update intervals Vec")
            .val
            .clone()
    }
}