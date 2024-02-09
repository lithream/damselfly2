use std::cmp::{max, min};
use crate::damselfly::consts::{_BLOCK_SIZE, DEFAULT_MEMORYSPAN, DEFAULT_OPERATION_LOG_SIZE, DEFAULT_ROW_LENGTH, DEFAULT_TIMESPAN};
use crate::damselfly::map_manipulator;
use crate::damselfly::memory_structs::{MemoryStatus, MemoryUpdate, MemoryUsage, NoHashMap};
use crate::damselfly::viewer::DamselflyViewer;

pub struct DamselflyController {
    pub viewer: DamselflyViewer,
    pub graph_highlight: usize,
    pub map_highlight: usize,
    pub timespan: (usize, usize),
    pub memory_span: (usize, usize),
    pub block_size: usize,
    pub timespan_freelook: bool,
    pub memoryspan_freelook: bool,

    pub row_length: usize,
}

impl DamselflyController {
    pub(crate) fn new() -> DamselflyController {
        DamselflyController {
            viewer: DamselflyViewer::new(),
            graph_highlight: 0,
            map_highlight: 0,
            timespan: (0, DEFAULT_TIMESPAN),
            memory_span: (0, DEFAULT_MEMORYSPAN),
            block_size: _BLOCK_SIZE,
            timespan_freelook: false,
            memoryspan_freelook: false,
            row_length: DEFAULT_ROW_LENGTH,
        }
    }

    pub fn get_timespan(&self) -> (usize, usize) {
        self.timespan
    }
    pub fn get_memoryspan(&self) -> (usize, usize) { self.memory_span }
    /// Shifts timespan to the right.
    ///
    /// The absolute distance shifted is computed by multiplying units with the
    /// current timespan.
    ///
    pub fn shift_timespan_right(&mut self, units: usize) -> bool {
        let right = &mut self.timespan.1;
        let left = &mut self.timespan.0;
        debug_assert!(*right > *left);
        let span = *right - *left;
        if span < DEFAULT_TIMESPAN { return false; }
        let absolute_shift = units * DEFAULT_TIMESPAN;

        let memory_usage_snapshots = self.viewer.count_memory_usage_snapshots();
        if *right + absolute_shift > memory_usage_snapshots - 1 {
            *right = memory_usage_snapshots - 1;
            *left = *right - DEFAULT_TIMESPAN;
            return false;
        }

        *right = min((*right).saturating_add(absolute_shift), memory_usage_snapshots - 1);
        *left = min((*left).saturating_add(absolute_shift), *right - DEFAULT_TIMESPAN);

        debug_assert!(right > left);
        true
    }

    /// Shifts timespan to the left.
    ///
    /// The absolute distance shifted is computed by multiplying units with the
    /// current timespan.
    ///
    pub fn shift_timespan_left(&mut self, units: usize) {
        self.timespan_freelook = true;
        let right = &mut self.timespan.1;
        let left = &mut self.timespan.0;
        debug_assert!(*right > *left);
        let span = *right - *left;
        let absolute_shift = units * span;

        *left = (*left).saturating_sub(absolute_shift);
        *right = *left + DEFAULT_TIMESPAN;
        debug_assert!(*right > *left);
    }

    pub fn shift_timespan_to_beginning(&mut self) {
        let old_timespan = self.timespan;
        self.timespan.0 = 0;
        self.timespan.1 = old_timespan.1 - old_timespan.0;
    }

    /// Shifts timespan to include the most recent data.
    pub fn shift_timespan_to_end(&mut self) {
        let old_timespan = self.timespan;
        self.timespan.1 = self.viewer.get_total_operations() - 1;
        self.timespan.0 = self.timespan.1 - (old_timespan.1 - old_timespan.0);
    }

    /// Locks the timespan, forcing it to automatically follow along as new data streams in.
    pub fn lock_timespan(&mut self) {
        let current_span = max(DEFAULT_TIMESPAN, self.timespan.1 - self.timespan.0);
        self.timespan.1 = self.viewer.count_memory_usage_snapshots().saturating_sub(1);
        self.timespan.0 = self.timespan.1.saturating_sub(current_span);
        self.timespan_freelook = false;
    }

    pub fn extend_span_to_cover_all_data(&mut self) {
        self.timespan.0 = 0;
        self.timespan.1 = self.viewer.get_total_operations() - 1;
    }

    pub fn unlock_timespan(&mut self) {
        self.timespan_freelook = true;
    }

    pub fn update_graph_highlight(&mut self, new_highlight: usize) {
        self.graph_highlight = new_highlight;
    }

    pub fn shift_graph_highlight_right(&mut self) {
        if self.graph_highlight + self.timespan.0 == self.timespan.1 - 1 {
            self.shift_timespan_right(1);
            self.graph_highlight = 0;
        } else {
            self.graph_highlight =  (self.graph_highlight + 1).clamp(0, self.timespan.1 - self.timespan.0 - 1);
        }
    }

    pub fn shift_graph_highlight_left(&mut self) {
        if self.graph_highlight + self.timespan.0 == self.timespan.0 {
            self.shift_timespan_left(1);
            self.graph_highlight = DEFAULT_TIMESPAN - 1;
        } else {
            self.graph_highlight = self.graph_highlight.saturating_sub(1);
        }
    }

    pub fn get_graph_highlight_absolute(&self) -> usize {
        self.graph_highlight + self.timespan.0
    }

    pub fn get_current_memory_usage(&self) -> MemoryUsage {
        self.viewer.get_memory_usage_at(self.graph_highlight + self.timespan.0)
    }

    pub fn get_current_memory_usage_graph(&self) -> Vec<[f64; 2]> {
        self.viewer.get_memory_usage_view(self.timespan.0, self.timespan.1)
    }

    pub fn get_full_memory_usage_graph(&self) -> Vec<[f64; 2]> {
        let end = self.viewer.get_total_operations() - 1;
        self.viewer.get_memory_usage_view(0, end)
    }

    pub fn get_current_map_state(&mut self) -> (NoHashMap<usize, MemoryStatus>, Option<&MemoryUpdate>) {
        if !self.memoryspan_freelook {
            let current_operation = self.get_current_memory_usage().latest_operation;
            self.snap_memoryspan_to_address(current_operation);
        }
        // unnecessary since get_map_state converts back from absolute to logical, todo optimise later
        self.viewer.get_map_state(self.timespan.0 + self.graph_highlight,
                                  map_manipulator::logical_to_absolute(self.memory_span.0, self.block_size),
                                  map_manipulator::logical_to_absolute(self.memory_span.1, self.block_size),
                                  self.block_size)
    }

    pub fn get_current_operation_log(&self) -> &[MemoryUpdate] {
        let end = min(self.graph_highlight + self.timespan.0, self.viewer.get_total_operations() - 1);
        let start = end.saturating_sub(DEFAULT_OPERATION_LOG_SIZE);
        self.viewer.get_operation_log(start, end)
    }

    pub fn snap_memoryspan_to_address(&mut self, absolute_address: usize) {
        let mut new_map_span = self.memory_span;
        let relative_address = map_manipulator::absolute_to_logical(absolute_address, self.block_size);
        let address_of_row = map_manipulator::get_address_of_row(self.row_length, relative_address);
        if relative_address >= self.memory_span.1 {
            new_map_span.0 = address_of_row.saturating_sub(DEFAULT_MEMORYSPAN / 2);
            new_map_span.1 = new_map_span.0 + DEFAULT_MEMORYSPAN;
        } else if relative_address < self.memory_span.0 {
            new_map_span.1 = relative_address + DEFAULT_MEMORYSPAN / 2;
            new_map_span.0 = new_map_span.1.saturating_sub(DEFAULT_MEMORYSPAN);
        }
        self.map_highlight = relative_address;
        self.memory_span = new_map_span;
    }

    pub fn get_current_operation(&self) -> Option<&MemoryUpdate> {
        self.viewer.get_operation_address_at_time(self.graph_highlight)
    }
}