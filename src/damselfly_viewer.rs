pub mod instruction;
pub mod consts;
use std::cmp::{max, min};
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::rc::Rc;
use std::time::Duration;
use nohash_hasher::BuildNoHashHasher;
use owo_colors::OwoColorize;
use crate::damselfly_viewer::consts::{DEFAULT_BLOCK_SIZE, DEFAULT_TIMESPAN, MAP_CACHE_SIZE};
use crate::damselfly_viewer::instruction::Instruction;
use crate::memory::{MemoryStatus, MemoryUpdate};
use crate::map_manipulator::MapManipulator;


pub type NoHashMap<K, V> = HashMap<K, V, BuildNoHashHasher<K>>;

#[derive(Debug, Default, Clone)]
pub struct MemoryUsage {
    pub memory_used_absolute: usize,
    pub total_memory: usize,
    pub blocks: usize,
}

pub struct Ranking {
    function: String,
    memory_used: usize,
}

pub struct Leaderboard {
    heap: BinaryHeap<Ranking>
}

#[derive(Debug)]
pub struct DamselflyViewer {
    instruction_rx: crossbeam_channel::Receiver<Instruction>,
    timespan: (usize, usize),
    timespan_is_unlocked: bool,
    memoryspan: (usize, usize),
    memory_usage_snapshots: Vec<MemoryUsage>,
    operation_history: Vec<MemoryUpdate>,
    operation_history_map: HashMap<usize, MemoryUpdate>,
    memory_map: NoHashMap<usize, MemoryStatus>,
    memory_map_snapshots: Vec<NoHashMap<usize, MemoryStatus>>,
    current_memory_map_snapshot: NoHashMap<usize, MemoryStatus>,
    min_address: usize,
    max_address: usize,
    max_usage: usize,
    max_blocks: usize,
}

impl DamselflyViewer {
    pub fn new(instruction_rx: crossbeam_channel::Receiver<Instruction>) -> DamselflyViewer {
        DamselflyViewer {
            instruction_rx,
            timespan: (0, DEFAULT_TIMESPAN),
            timespan_is_unlocked: true,
            memoryspan: (0, consts::DEFAULT_MEMORYSPAN),
            memory_usage_snapshots: Vec::new(),
            operation_history: Vec::new(),
            operation_history_map: HashMap::new(),
            memory_map: NoHashMap::default(),
            memory_map_snapshots: Vec::new(),
            current_memory_map_snapshot: NoHashMap::default(),
            min_address: usize::MAX,
            max_address: usize::MIN,
            max_usage: usize::MIN,
            max_blocks: usize::MIN,
        }
    }

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

        if *right + absolute_shift > self.memory_usage_snapshots.len() - 1 {
            *right = self.memory_usage_snapshots.len() - 1;
            *left = *right - DEFAULT_TIMESPAN;
            return false;
        }

        *right = min((*right).saturating_add(absolute_shift), self.memory_usage_snapshots.len() - 1);
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
        self.timespan_is_unlocked = true;
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

    pub fn gulp_channel(&mut self) {
        // bookkeeping
        let mut max_usage = usize::MIN;
        let mut max_blocks = usize::MIN;
        let mut start_addresses = HashSet::new();
        let mut end_addresses = HashSet::new();

        let mut counter = 0;
        let mut blocks = 0;

        let mut current_map_snapshot = NoHashMap::default();
        while let Ok(instruction) = self.instruction_rx.recv_timeout(Duration::from_nanos(1)) {
            let instruction_string = instruction.get_operation().to_string();
            eprintln!("Processing instruction {}: {}", counter.cyan(), instruction_string);
            if counter % MAP_CACHE_SIZE == 0 {
                eprintln!("Caching map...");
                self.memory_map_snapshots.push(current_map_snapshot.clone());
            }
            counter += 1;

            let modified_blocks = self.count_modified_bytes(&instruction.get_operation(), &self.operation_history_map);
            let mut usage = self.calculate_memory_usage(&instruction, modified_blocks);
            Self::count_blocks_in_memory(&instruction.get_operation(), &mut start_addresses, &mut end_addresses, &mut blocks);
            usage.blocks = blocks;

            max_usage = max(usage.memory_used_absolute, max_usage);
            max_blocks = max(usage.blocks, max_blocks);

            self.memory_usage_snapshots.push(usage);
            match instruction.get_operation() {
                MemoryUpdate::Allocation(address, size, callstack) =>
                    MapManipulator::allocate_memory(&mut current_map_snapshot, address, size, callstack),
                MemoryUpdate::Free(address, callstack) => {
                    MapManipulator::free_memory(&mut current_map_snapshot, address, callstack);
                }
            }
            self.log_operation(instruction);
        }

        self.max_usage = max_usage;
        self.max_blocks = max_blocks;
    }

    pub fn calculate_memory_usage(&mut self, instruction: &Instruction, modified_blocks: usize) -> MemoryUsage {
        unsafe {
            // calculate total usage
            static mut MEMORY_USED_ABSOLUTE: usize = 0;
            match instruction.get_operation() {
                MemoryUpdate::Allocation(..) => MEMORY_USED_ABSOLUTE += modified_blocks,
                MemoryUpdate::Free(..) => MEMORY_USED_ABSOLUTE -= modified_blocks,
            }

            MemoryUsage {
                memory_used_absolute: MEMORY_USED_ABSOLUTE,
                total_memory: consts::DEFAULT_MEMORY_SIZE,
                blocks: 0,
            }
        }
    }

    fn count_modified_bytes(&self, operation: &MemoryUpdate, operation_history_map: &HashMap<usize, MemoryUpdate>) -> usize {
        let mut modified_bytes = 0;
        match operation {
            MemoryUpdate::Allocation(_, size, _) => {
                modified_bytes = *size;
            },
            MemoryUpdate::Free(address, ..) => {
                if let Some(MemoryUpdate::Allocation(_, size, _)) = operation_history_map.get(address) {
                    modified_bytes = *size;
                }
            }
        }

        modified_bytes
    }

    fn count_blocks_in_memory(latest_operation: &MemoryUpdate, start_addresses: &mut HashSet<usize>, end_addresses: &mut HashSet<usize>, blocks: &mut usize) {
        let mut count_blocks = |operation: &MemoryUpdate| {
            let start_address;
            let end_address;
            match operation {
                MemoryUpdate::Allocation(address, size, ..) => {
                    start_address = *address;
                    end_address = start_address + size;
                    if !start_addresses.contains(&end_address) && !end_addresses.contains(&start_address) {
                        // distinct block
                        *blocks += 1;
                    } else if start_addresses.contains(&end_address) && end_addresses.contains(&start_address) {
                        // block connects two other blocks
                        *blocks -= 1;
                    }
                    // block is prepended or appended to another block, so no fragmentation results
                    start_addresses.insert(start_address);
                    end_addresses.insert(end_address);
                }
                MemoryUpdate::Free(start_address, ..) => {
                    let mut sorted_end_addresses: Vec<usize> = end_addresses.iter().cloned().collect();
                    sorted_end_addresses.sort_unstable();
                    let end_address = sorted_end_addresses.iter().find(|address| **address > *start_address)
                        .expect("[DamselflyViewer::count_blocks_in_memory]: Failed to find end address corresponding to free");
                    if end_addresses.contains(start_address) && start_addresses.contains(end_address) {
                        // fragment created
                        *blocks += 1;
                    } else if !end_addresses.contains(start_address) && !start_addresses.contains(end_address) {
                        // lone block removed
                        *blocks -= 1;
                    }
                    start_addresses.remove(start_address);
                    end_addresses.remove(end_address);
                }
            };
        };
        count_blocks(latest_operation);
    }

    fn update_memory_map(&mut self, instruction: &Instruction) -> usize {
        match instruction.get_operation() {
            MemoryUpdate::Allocation(address, size, callstack) => {
                MapManipulator::allocate_memory(&mut self.memory_map, address, size, Rc::clone(&callstack));
                size
            }
            MemoryUpdate::Free(address, callstack) => {
                MapManipulator::free_memory(&mut self.memory_map, address, Rc::clone(&callstack))
            }
        }
    }

    fn update_min_max_address(&mut self, address: usize) {
        self.min_address = min(address, self.min_address);
        self.max_address = max(address, self.max_address);
    }

    fn log_operation(&mut self, instruction: Instruction) {
        let operation = instruction.get_operation();
        let op_address;
        match operation {
            MemoryUpdate::Allocation(address, _, _) => {
                op_address = address;
                self.update_min_max_address(address)
            },
            MemoryUpdate::Free(address, _) => {
                op_address = address;
                self.update_min_max_address(address)
            },
        }
        self.operation_history.push(operation.clone());
        self.operation_history_map.insert(op_address, operation);
    }

    pub fn get_address_bounds(&self) -> (usize, usize) {
        (self.min_address, self.max_address)
    }

    pub fn get_memory_usage(&self) -> MemoryUsage {
        let memory_usage = self.memory_usage_snapshots.last();
        match memory_usage {
            None => {
                MemoryUsage{
                    memory_used_absolute: 0,
                    total_memory: consts::DEFAULT_MEMORY_SIZE,
                    blocks: 0,
                }
            }
            Some(memory_usage) => (*memory_usage).clone()
        }
    }

    pub fn get_memory_usage_at(&self, time: usize) -> MemoryUsage {
        if let Some(memory_usage) = self.memory_usage_snapshots.get(time) {
            return memory_usage.clone();
        }
        MemoryUsage{
            memory_used_absolute: 0,
            total_memory: consts::DEFAULT_MEMORY_SIZE,
            blocks: 0,
        }
    }

    pub fn get_memory_usage_view(&self) -> Vec<(f64, f64)> {
        let mut vector = Vec::new();
        for i in self.timespan.0..min(self.memory_usage_snapshots.len(), self.timespan.1) {
            let memory_used_absolute = self.memory_usage_snapshots.get(i)
                .expect("[DamselflyViewer::get_memory_usage_view]: Error getting timestamp from memory_usage_snapshots")
                .memory_used_absolute;
            let memory_used_percentage = memory_used_absolute as f64 * 100.0 / self.max_usage as f64;
            vector.push(((i - self.timespan.0) as f64, memory_used_percentage));
        }
        vector
    }

    pub fn get_latest_map_state(&self) -> (NoHashMap<usize, MemoryStatus>, Option<&MemoryUpdate>) {
        (self.memory_map.clone(), self.operation_history.get(self.get_total_operations().saturating_sub(1)))
    }

    pub fn get_map_state(&mut self, time: usize, span_start: usize, span_end: usize) -> (NoHashMap<usize, MemoryStatus>, Option<&MemoryUpdate>) {
        let starting_snapshot = time / MAP_CACHE_SIZE;
        let mut map =
            Self::clone_map_partial(self.memory_map_snapshots.get(starting_snapshot).unwrap(),
                                    MapManipulator::scale_address_down(span_start),
                                    MapManipulator::scale_address_down(span_end));

        for cur_time in starting_snapshot * MAP_CACHE_SIZE..=time {
            if let Some(operation) = self.operation_history.get(cur_time) {
                match operation {
                    MemoryUpdate::Allocation(absolute_address, size, callstack) => {
                        if let Some((fill_start, bytes_to_fill)) = Self::bytes_allocated_within_span(*absolute_address, *size, span_start, span_end) {
                            Self::allocate_memory(&mut map, fill_start, bytes_to_fill, Rc::clone(callstack));
                        }
                    }
                    MemoryUpdate::Free(absolute_address, callstack) => {
                        if let Some((free_start, bytes_to_free)) = Self::bytes_freed_within_span(*absolute_address, span_start, span_end, &map) {
                            Self::free_memory_manual(&mut map, free_start, bytes_to_free, Rc::clone(callstack));
                        }
                    }
                }
            }
        }
        (map, self.operation_history.get(time))
    }

    fn clone_map_partial(map: &NoHashMap<usize, MemoryStatus>, span_start: usize, span_end: usize) -> NoHashMap<usize, MemoryStatus> {
        let mut new_map = NoHashMap::default();
        for block in span_start..=span_end {
            if let Some(status) = map.get(&block) {
                new_map.insert(block, status.clone());
            }
        }
        new_map
    }

    pub fn bytes_allocated_within_span(op_address: usize, size: usize, span_start: usize, span_end: usize) -> Option<(usize, usize)> {
        let span_size = (span_end - span_start) * DEFAULT_BLOCK_SIZE;
        // allocation starts before span and does not continue into it
        if op_address + size < span_start {
            return None;
        }
        // allocation starts before span, but continues into it
        if op_address < span_start {
            let bytes_to_fill = min(span_size, op_address + size - span_start);
            return Some((span_start, bytes_to_fill));
        }
        // allocation starts inside span
        if op_address >= span_start && op_address <= span_end {
            let bytes_to_fill = min(size, span_size);
            return Some((op_address, bytes_to_fill));
        }
        // allocation starts after span
        None
    }

    pub fn bytes_freed_within_span(parent_block: usize, span_start: usize, span_end: usize, map: &NoHashMap<usize, MemoryStatus>) -> Option<(usize, usize)> {
        let start = max(parent_block, span_start);
        let mut current_address = start;
        // If free happens after the span, it need not be reflected within the span
        if parent_block > span_end {
            return None;
        }

        while let Some(block_status) = MapManipulator::view_memory(map, current_address) {
            match block_status {
                MemoryStatus::Allocated(allocated_parent_block, _, _) => {
                    if *allocated_parent_block != parent_block {
                        return Some((start, current_address - start));
                    }
                }
                MemoryStatus::PartiallyAllocated(p_allocated_parent_block, _) => {
                    if *p_allocated_parent_block != parent_block {
                        return Some((start, current_address - start));
                    }
                }
                MemoryStatus::Free(_) => {
                    return Some((start, current_address - start));
                }
            }
            current_address += 1;
            if current_address > span_end {
                return Some((start, current_address - start));
            }
        }
        Some((start, current_address - start))
    }

    pub fn count_adjacent_allocated_blocks(start_address: usize, end_address: usize, parent_block_address: usize, map: &NoHashMap<usize, MemoryStatus>) -> usize {
        let mut current_address = start_address;
        current_address += 1;
        while let Some(next_block) = MapManipulator::view_memory(map, current_address) {
            match next_block {
                MemoryStatus::Allocated(allocated_parent_block, ..) => {
                    if *allocated_parent_block != parent_block_address {
                        return (current_address - start_address) / DEFAULT_BLOCK_SIZE;
                    }
                }
                MemoryStatus::PartiallyAllocated(p_allocated_parent_block, ..) => {
                    if *p_allocated_parent_block != parent_block_address {
                        return (current_address - start_address) / DEFAULT_BLOCK_SIZE;
                    }
                }
                MemoryStatus::Free(_) => return (current_address - start_address) / DEFAULT_BLOCK_SIZE,
            }
            current_address += 1;
            if current_address > end_address {
                return (current_address - start_address) / DEFAULT_BLOCK_SIZE;
            }
        }
        (current_address - start_address) / DEFAULT_BLOCK_SIZE
    }

    fn free_memory(map: &mut NoHashMap<usize, MemoryStatus>, absolute_address: usize, callstack: Rc<String>) {
        let scaled_address = MapManipulator::scale_address_down(absolute_address);
        if scaled_address * DEFAULT_BLOCK_SIZE != absolute_address {
            panic!("[DamselflyViewer::free_memory]: Block arithmetic error");
        }
        let mut offset = 0;
        if map.get(&scaled_address).is_none() {
            map.insert(scaled_address, MemoryStatus::Free(Rc::clone(&callstack)));
        }
        while let Some(status) = map.get(&(scaled_address + offset)) {
            match status {
                MemoryStatus::Allocated(parent_block, _, _) => {
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
            map.insert(scaled_address + offset, MemoryStatus::Free(Rc::clone(&callstack)));
            offset += 1;
        }
    }

    fn free_memory_manual(map: &mut NoHashMap<usize, MemoryStatus>, absolute_address: usize, bytes: usize, callstack: Rc<String>) {
        let scaled_address = MapManipulator::scale_address_down(absolute_address);
        if scaled_address * DEFAULT_BLOCK_SIZE != absolute_address {
            panic!("[DamselflyViewer::free_memory]: Block arithmetic error");
        }
        let blocks = bytes / DEFAULT_BLOCK_SIZE;
        for i in 0..blocks {
            map.insert(scaled_address + i, MemoryStatus::Free(Rc::clone(&callstack)));
        }
    }

    pub fn allocate_memory(map: &mut NoHashMap<usize, MemoryStatus>, absolute_address: usize, bytes: usize, callstack: Rc<String>) {
        let scaled_address = MapManipulator::scale_address_down(absolute_address);
        if bytes == 0 {
            return;
        }
        if bytes < DEFAULT_BLOCK_SIZE && bytes > 0 {
            map.insert(scaled_address, MemoryStatus::PartiallyAllocated(absolute_address, callstack));
            return;
        }

        let full_blocks = bytes / DEFAULT_BLOCK_SIZE;
        for block_count in 0..full_blocks {
            map.insert(scaled_address + block_count, MemoryStatus::Allocated(absolute_address, bytes, Rc::clone(&callstack)));
        }

        if (full_blocks * DEFAULT_BLOCK_SIZE) < bytes {
            map.insert(scaled_address + full_blocks, MemoryStatus::PartiallyAllocated(absolute_address, Rc::clone(&callstack)));
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

    pub fn get_operation_log_span(&self, mut start: usize, mut end: usize) -> &[MemoryUpdate] {
        let operations = self.operation_history.len();
        start = start.clamp(0, operations - 1);
        end = end.clamp(0, operations - 1);
        if self.operation_history.get(start).is_none() || self.operation_history.get(end.saturating_sub(1)).is_none() {
            return &[];
        }
        &self.operation_history[start..=end]
    }
    
    pub fn is_timespan_locked(&self) -> bool {
        !self.timespan_is_unlocked
    }

}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::iter::Map;
    use std::rc::Rc;
    use crate::damselfly_viewer::{DamselflyViewer, consts};
    use crate::damselfly_viewer::consts::{DEFAULT_BINARY_PATH, DEFAULT_GADDR2LINE_PATH, DEFAULT_TIMESPAN};
    use crate::damselfly_viewer::instruction::Instruction;
    use crate::map_manipulator::MapManipulator;
    use crate::memory::{MemorySysTraceParser, MemoryUpdate};

    fn initialise_viewer() -> (DamselflyViewer, MemorySysTraceParser) {
        let (memory_stub, instruction_rx) = MemorySysTraceParser::new();
        let damselfly_viewer = DamselflyViewer::new(instruction_rx);
        (damselfly_viewer, memory_stub)
    }

    #[test]
    fn shift_timespan() {
        let (mut damselfly_viewer, mut mst_parser) = initialise_viewer();
        let log = std::fs::read_to_string(consts::TEST_LOG_PATH).unwrap();
        mst_parser.parse_log(log, DEFAULT_BINARY_PATH);
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
        mst_parser.parse_log(log, DEFAULT_BINARY_PATH);
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
        mst_parser.parse_log(log, DEFAULT_BINARY_PATH);
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

    #[test]
    fn count_adjacent_blocks_allocation_outside_span_test() {
        let (mut damselfly_viewer, _ ) = initialise_viewer();
        MapManipulator::allocate_memory(&mut damselfly_viewer.memory_map, 0, 128, Rc::new(String::from("test")));
        let count = DamselflyViewer::count_adjacent_allocated_blocks(256, 128, 0, &damselfly_viewer.memory_map);
        assert_eq!(count, 0);
    }

    #[test]
    fn count_adjacent_blocks_allocation_flowing_into_span_test() {
        let (mut damselfly_viewer, _ ) = initialise_viewer();
        MapManipulator::allocate_memory(&mut damselfly_viewer.memory_map, 0, 128, Rc::new(String::from("test")));
        let count = DamselflyViewer::count_adjacent_allocated_blocks(64, 128, 0, &damselfly_viewer.memory_map);
        assert_eq!(count, 16);
    }

    #[test]
    fn count_blocks_in_memory_one_block_test() {
        let instruction = Instruction::new(0, MemoryUpdate::Allocation(0, 128, Rc::new(String::from("test"))));
        let mut start_addresses = HashSet::new();
        let mut end_addresses = HashSet::new();
        let mut blocks = 0;
        DamselflyViewer::count_blocks_in_memory(&instruction.get_operation(), &mut start_addresses, &mut end_addresses, &mut blocks);
        assert_eq!(blocks, 1);
    }

    #[test]
    fn count_blocks_in_memory_multiple_blocks_test() {
        let mut operation_history: Vec<MemoryUpdate> = Vec::new();
        let mut blocks = 0;
        // one block
        let mut start_addresses = HashSet::new();
        let mut end_addresses = HashSet::new();
        let latest_instruction = Instruction::new(0, MemoryUpdate::Allocation(0, 128, Rc::new(String::from("test"))));
        operation_history.push(latest_instruction.get_operation());
        DamselflyViewer::count_blocks_in_memory(&latest_instruction.get_operation(), &mut start_addresses, &mut end_addresses, &mut blocks);
        assert_eq!(blocks, 1);

        // two blocks
        let latest_instruction = Instruction::new(1, MemoryUpdate::Allocation(256, 128, Rc::new(String::from("test"))));
        DamselflyViewer::count_blocks_in_memory(&latest_instruction.get_operation(), &mut start_addresses, &mut end_addresses, &mut blocks);
        assert_eq!(blocks, 2);
        operation_history.push(latest_instruction.get_operation());

        // still two blocks
        let latest_instruction = Instruction::new(2, MemoryUpdate::Allocation(128, 64, Rc::new(String::from("test"))));
        DamselflyViewer::count_blocks_in_memory(&latest_instruction.get_operation(), &mut start_addresses, &mut end_addresses, &mut blocks);
        assert_eq!(blocks, 2);
        operation_history.push(latest_instruction.get_operation());

        // merge into one block
        let latest_instruction = Instruction::new(3, MemoryUpdate::Allocation(192, 64, Rc::new(String::from("test"))));
        operation_history.push(latest_instruction.get_operation());
        DamselflyViewer::count_blocks_in_memory(&latest_instruction.get_operation(), &mut start_addresses, &mut end_addresses, &mut blocks);
        assert_eq!(blocks, 1);
        operation_history.push(latest_instruction.get_operation());

        // split into two blocks
        let latest_instruction = Instruction::new(4, MemoryUpdate::Free(192, Rc::new(String::from("test"))));
        DamselflyViewer::count_blocks_in_memory(&latest_instruction.get_operation(), &mut start_addresses, &mut end_addresses, &mut blocks);
        assert_eq!(blocks, 2);
        operation_history.push(latest_instruction.get_operation());

        // remove one block, leaving one
        let latest_instruction = Instruction::new(5, MemoryUpdate::Free(256, Rc::new(String::from("test"))));
        DamselflyViewer::count_blocks_in_memory(&latest_instruction.get_operation(), &mut start_addresses, &mut end_addresses, &mut blocks);
        assert_eq!(blocks, 1);
        operation_history.push(latest_instruction.get_operation());

        /*
        // zero blocks
        let latest_instruction = Instruction::new(2, MemoryUpdate::Free(64, Rc::new(String::from("test"))));
        assert_eq!(DamselflyViewer::count_blocks_in_memory(&latest_instruction.get_operation(), &operation_history), 1);
         */
    }
}

