pub mod instruction;
pub mod consts;

use std::cmp::{max, min};
use std::collections::HashMap;
use std::sync::{mpsc};
use std::time::Duration;
use log::debug;
use crate::damselfly_viewer::consts::{DEFAULT_BLOCK_SIZE, DEFAULT_TIMESPAN};
use crate::damselfly_viewer::instruction::Instruction;
use crate::memory::{MemoryStatus, MemoryUpdate};
use crate::map_manipulator::MapManipulator;


#[derive(Debug, Default, Clone)]
pub struct MemoryUsage {
    pub memory_used_percentage: f64,
    pub memory_used_absolute: f64,
    pub total_memory: usize
}

#[derive(Debug)]
pub struct DamselflyViewer {
    instruction_rx: mpsc::Receiver<Instruction>,
    timespan: (usize, usize),
    timespan_is_unlocked: bool,
    memoryspan: (usize, usize),
    memoryspan_is_unlocked: bool,
    memory_usage_snapshots: Vec<MemoryUsage>,
    operation_history: Vec<MemoryUpdate>,
    memory_map: HashMap<usize, MemoryStatus>,
}

impl DamselflyViewer {
    pub fn new(instruction_rx: mpsc::Receiver<Instruction>) -> DamselflyViewer {
        DamselflyViewer {
            instruction_rx,
            timespan: (0, DEFAULT_TIMESPAN),
            timespan_is_unlocked: true,
            memoryspan: (0, consts::DEFAULT_MEMORYSPAN),
            memoryspan_is_unlocked: false,
            memory_usage_snapshots: Vec::new(),
            operation_history: Vec::new(),
            memory_map: HashMap::new(),
        }
    }

    /// Shifts timespan to the right.
    ///
    /// The absolute distance shifted is computed by multiplying units with the
    /// current timespan.
    ///
    pub fn shift_timespan_right(&mut self, units: usize) {
        let right = &mut self.timespan.1;
        let left = &mut self.timespan.0;
        debug_assert!(*right > *left);
        let span = *right - *left;
        if span < DEFAULT_TIMESPAN { return; }
        let absolute_shift = units * span;

        *right = min((*right).saturating_add(absolute_shift), self.memory_usage_snapshots.len() - 1);
        *left = min((*left).saturating_add(absolute_shift), *right - span);
        debug_assert!(right > left);
    }

    /// Shifts timespan to the left.
    ///
    /// The absolute distance shifted is computed by multiplying units with the
    /// current timespan.
    ///
    pub fn shift_timespan_left(&mut self, units: usize) {
        self.timespan_is_unlocked = true;
        let right = &mut self.timespan.1;
        let left = &mut self.timespan.0;
        debug_assert!(*right > *left);
        let span = *right - *left;
        let absolute_shift = units * span;

        *left = (*left).saturating_sub(absolute_shift);
        *right = max((*right).saturating_sub(absolute_shift), *left + span);
        debug_assert!(*right > *left);
    }

    pub fn shift_timespan_to_beginning(&mut self) {
        let span = self.get_timespan();
        self.timespan.0 = 0;
        self.timespan.1 = span.1 - span.0;
    }

    /// Shifts timespan to include the most recent data.
    pub fn shift_timespan_to_end(&mut self) {
        let span = self.get_timespan();
        self.timespan.1 = self.get_total_operations() - 1;
        self.timespan.0 = self.timespan.1 - (span.1 - span.0);
    }

    /// Locks the timespan, forcing it to automatically follow along as new data streams in.
    pub fn lock_timespan(&mut self) {
        let current_span = max(consts::DEFAULT_TIMESPAN, self.timespan.1 - self.timespan.0);
        self.timespan.1 = self.memory_usage_snapshots.len().saturating_sub(1);
        self.timespan.0 = self.timespan.1.saturating_sub(current_span);
        self.timespan_is_unlocked = false;
    }

    pub fn unlock_timespan(&mut self) {
        self.timespan_is_unlocked = true;
    }

    pub fn update(&mut self) {
        let update = self.instruction_rx.recv();
        match update {
            Ok(instruction) => {
                self.update_memory_map(&instruction);
                self.calculate_memory_usage();
                self.log_operation(instruction);
            }
            Err(_) => {
                debug!("[damselfly_viewer::update]: Snapshot channel hung up.");
                return;
            }
        }


        if !self.timespan_is_unlocked {
            self.timespan.1 += 1;
            if self.timespan.1 > consts::DEFAULT_TIMESPAN {
                self.timespan.0 += 1;
            }
        }

        if !self.memoryspan_is_unlocked {
            // do nothing, memoryspan locking in tui
        }
    }

    pub fn gulp_channel(&mut self) {
        while let Ok(instruction) = self.instruction_rx.recv_timeout(Duration::from_nanos(1)) {
            self.update_memory_map(&instruction);
            self.calculate_memory_usage();
            self.log_operation(instruction);
        }
    }

    pub fn calculate_memory_usage(&mut self) {
        let mut memory_used_absolute: f64 = 0.0;
        for (_, status) in self.memory_map.iter() {
            match status {
                MemoryStatus::Allocated(_, _) => memory_used_absolute += 1.0,
                MemoryStatus::PartiallyAllocated(_, _) => memory_used_absolute += 0.5,
                MemoryStatus::Free(_) => {}
            }
        }

        let memory_usage = MemoryUsage {
            memory_used_percentage: (memory_used_absolute / consts::DEFAULT_MEMORY_SIZE as f64) * 100.0,
            memory_used_absolute,
            total_memory: consts::DEFAULT_MEMORY_SIZE,
        };

        self.memory_usage_snapshots.push(memory_usage);
    }

    fn update_memory_map(&mut self, instruction: &Instruction) {
        match instruction.get_operation() {
            MemoryUpdate::Allocation(address, size, callstack) =>
                MapManipulator::allocate_memory(&mut self.memory_map, address, size, callstack.clone()),
            MemoryUpdate::Free(address, callstack) =>
                MapManipulator::free_memory(&mut self.memory_map, address, callstack.clone()),
        };
    }

    fn log_operation(&mut self, instruction: Instruction) {
        self.operation_history.push(instruction.get_operation());
    }

    pub fn get_memory_usage(&self) -> MemoryUsage {
        let memory_usage = self.memory_usage_snapshots.last();
        match memory_usage {
            None => {
                MemoryUsage{
                    memory_used_percentage: 0.0,
                    memory_used_absolute: 0.0,
                    total_memory: consts::DEFAULT_MEMORY_SIZE,
                }
            }
            Some(memory_usage) => (*memory_usage).clone()
        }
    }

    pub fn get_memory_usage_view(&self) -> Vec<(f64, f64)> {
        let mut vector = Vec::new();
        for i in self.timespan.0..min(self.memory_usage_snapshots.len(), self.timespan.1) {
            vector.push(((i - self.timespan.0) as f64, self.memory_usage_snapshots.get(i)
                .expect("[DamselflyViewer::get_memory_usage_view]: Error getting timestamp {i} from memory_usage_snapshots")
                .memory_used_percentage));
        }
        vector
    }

    pub fn get_latest_map_state(&self) -> (HashMap<usize, MemoryStatus>, Option<&MemoryUpdate>) {
        (self.memory_map.clone(), self.operation_history.get(self.get_total_operations().saturating_sub(1)))
    }

    pub fn get_map_state(&self, time: usize) -> (HashMap<usize, MemoryStatus>, Option<&MemoryUpdate>) {
        let mut map: HashMap<usize, MemoryStatus> = HashMap::new();
        let mut iter = self.operation_history.iter();
        for i in 0..=time {
            if let Some(operation) = iter.next() {
                match operation {
                    MemoryUpdate::Allocation(absolute_address, size, callstack) => {
                        Self::allocate_memory(&mut map, *absolute_address, *size, callstack);
                    }
                    MemoryUpdate::Free(absolute_address, callstack) => {
                        Self::free_memory(&mut map, *absolute_address, callstack);
                    }
                }
            }
        }
        (map, self.operation_history.get(time))
    }

    fn free_memory(map: &mut HashMap<usize, MemoryStatus>, absolute_address: usize, callstack: &str) {
        let scaled_address = absolute_address / DEFAULT_BLOCK_SIZE;
        if scaled_address * DEFAULT_BLOCK_SIZE != absolute_address {
            panic!("[DamselflyViewer::free_memory]: Block arithmetic error");
        }
        let mut offset = 0;
        if map.get(&scaled_address).is_none() {
            map.insert(scaled_address, MemoryStatus::Free(callstack.to_string()));
        }
        while let Some(status) = map.get(&(scaled_address + offset)) {
            match status {
                MemoryStatus::Allocated(parent_block, _) => {
                    if *parent_block != scaled_address {
                        return;
                    }
                }
                MemoryStatus::PartiallyAllocated(parent_block, _) => {
                    if *parent_block != scaled_address {
                        return;
                    }
                }
                MemoryStatus::Free(_) => return,
            }
            map.insert(scaled_address + offset, MemoryStatus::Free(callstack.to_string()));
            offset += 1;
        }
    }

    pub fn allocate_memory(map: &mut HashMap<usize, MemoryStatus>, absolute_address: usize, bytes: usize, callstack: &str) {
        let scaled_address = MapManipulator::scale_address_down(absolute_address);
        if bytes == 0 {
            return;
        }
        if bytes < DEFAULT_BLOCK_SIZE && bytes > 0 {
            map.insert(scaled_address / 4, MemoryStatus::PartiallyAllocated(scaled_address, callstack.to_string()));
            return;
        }

        let full_blocks = bytes / DEFAULT_BLOCK_SIZE;
        for block_count in 0..full_blocks {
            map.insert(scaled_address + block_count, MemoryStatus::Allocated(scaled_address, String::from(callstack)));
        }

        if (full_blocks * DEFAULT_BLOCK_SIZE) < bytes {
            map.insert(scaled_address + full_blocks, MemoryStatus::PartiallyAllocated(scaled_address, String::from(callstack)));
        }
    }

    pub fn get_operation_address_at_time(&self, time: usize) -> Option<&MemoryUpdate> {
        self.operation_history.get(time)
    }

    pub fn get_timespan(&self) -> (usize, usize) {
        self.timespan
    }

    pub fn get_memoryspan(&self) -> (usize, usize) {
        self.memoryspan
    }

    pub fn get_total_operations(&self) -> usize {
        self.memory_usage_snapshots.len()
    }

    pub fn get_operation_log_span(&self, start: usize, end: usize) -> &[MemoryUpdate] {
        if self.operation_history.get(start).is_none() || self.operation_history.get(end - 1).is_none() {
            return &[];
        }
        &self.operation_history[start..end]
    }
    
    pub fn is_timespan_locked(&self) -> bool {
        !self.timespan_is_unlocked
    }

}

#[cfg(test)]
mod tests {
    use crate::damselfly_viewer::{DamselflyViewer, consts::DEFAULT_MEMORY_SIZE, consts};
    use crate::damselfly_viewer::consts::DEFAULT_TIMESPAN;
    use crate::memory::{MemoryStatus, MemorySysTraceParser, MemoryUpdate};

    fn initialise_viewer() -> (DamselflyViewer, MemorySysTraceParser) {
        let (memory_stub, instruction_rx) = MemorySysTraceParser::new();
        let damselfly_viewer = DamselflyViewer::new(instruction_rx);
        (damselfly_viewer, memory_stub)
    }

    #[test]
    fn shift_timespan() {
        let (mut damselfly_viewer, mut mst_parser) = initialise_viewer();
        let log = std::fs::read_to_string(consts::TEST_LOG_PATH).unwrap();
        mst_parser.parse_log(log);
        damselfly_viewer.gulp_channel();
        damselfly_viewer.shift_timespan_to_beginning();
        assert_eq!(damselfly_viewer.timespan.0, 0);
        assert_eq!(damselfly_viewer.timespan.1, DEFAULT_TIMESPAN);
        damselfly_viewer.shift_timespan_left(1);
        assert_eq!(damselfly_viewer.timespan.0, 0);
        assert_eq!(damselfly_viewer.timespan.1, DEFAULT_TIMESPAN);
        damselfly_viewer.shift_timespan_right(1);
        assert_eq!(damselfly_viewer.timespan.0, DEFAULT_TIMESPAN);
        assert_eq!(damselfly_viewer.timespan.1, DEFAULT_TIMESPAN * 2);
    }

    #[test]
    fn shift_timespan_left_cap() {
        let (mut damselfly_viewer, mut mst_parser) = initialise_viewer();
        let log = std::fs::read_to_string(consts::TEST_LOG_PATH).unwrap();
        mst_parser.parse_log(log);
        damselfly_viewer.gulp_channel();
        damselfly_viewer.shift_timespan_to_beginning();
        damselfly_viewer.shift_timespan_left(3);
        assert_eq!(damselfly_viewer.timespan.0, 0);
        assert_eq!(damselfly_viewer.timespan.1, DEFAULT_TIMESPAN);
        damselfly_viewer.shift_timespan_right(1);
        assert_eq!(damselfly_viewer.timespan.0, DEFAULT_TIMESPAN);
        assert_eq!(damselfly_viewer.timespan.1, DEFAULT_TIMESPAN * 2);
        damselfly_viewer.shift_timespan_left(2);
        assert_eq!(damselfly_viewer.timespan.0, 0);
        assert_eq!(damselfly_viewer.timespan.1, DEFAULT_TIMESPAN);
    }

    #[test]
    fn shift_timespan_right() {
        let (mut damselfly_viewer, mut mst_parser) = initialise_viewer();
        let log = std::fs::read_to_string(consts::TEST_LOG_PATH).unwrap();
        mst_parser.parse_log(log);
        damselfly_viewer.gulp_channel();
        damselfly_viewer.shift_timespan_to_beginning();
        damselfly_viewer.shift_timespan_left(3);
        assert_eq!(damselfly_viewer.timespan.0, 0);
        assert_eq!(damselfly_viewer.timespan.1, DEFAULT_TIMESPAN);
        damselfly_viewer.shift_timespan_right(1);
        assert_eq!(damselfly_viewer.timespan.0, DEFAULT_TIMESPAN);
        assert_eq!(damselfly_viewer.timespan.1, DEFAULT_TIMESPAN * 2);
        damselfly_viewer.shift_timespan_to_end();
        damselfly_viewer.shift_timespan_right(2);
        let length = damselfly_viewer.memory_usage_snapshots.len();
        assert_eq!(damselfly_viewer.timespan.0, length - DEFAULT_TIMESPAN - 1);
        assert_eq!(damselfly_viewer.timespan.1, length - 1);
    }
}

