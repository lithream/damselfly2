use std::cmp::{max, min};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use owo_colors::OwoColorize;
use crate::damselfly::{consts, map_manipulator};
use crate::damselfly::consts::{DEFAULT_BLOCK_SIZE, MAP_CACHE_SIZE};
use crate::damselfly::instruction::Instruction;
use crate::damselfly::memory_structs::{MemoryStatus, MemoryUpdate, MemoryUsage, NoHashMap};


#[derive(Debug, Default)]
pub struct DamselflyViewer {
    instructions: Vec<Instruction>,
    memory_usage_snapshots: Vec<MemoryUsage>,
    operation_history: Vec<MemoryUpdate>,
    operation_history_map: HashMap<usize, MemoryUpdate>,
    memory_map: NoHashMap<usize, MemoryStatus>,
    memory_map_snapshots: Vec<NoHashMap<usize, MemoryStatus>>,
    min_address: usize,
    max_address: usize,
    max_usage: usize,
    max_blocks: usize,
}

impl DamselflyViewer {
    pub fn new() -> DamselflyViewer {
        DamselflyViewer {
            instructions: Vec::new(),
            memory_usage_snapshots: Vec::new(),
            operation_history: Vec::new(),
            operation_history_map: HashMap::new(),
            memory_map: NoHashMap::default(),
            memory_map_snapshots: Vec::new(),
            min_address: usize::MAX,
            max_address: usize::MIN,
            max_usage: usize::MIN,
            max_blocks: usize::MIN,
        }
    }

    pub fn count_memory_usage_snapshots(&self) -> usize {
        self.memory_usage_snapshots.len()
    }

    pub fn load_instructions(&mut self, instructions: Vec<Instruction>) {
        // bookkeeping
        let mut max_usage = usize::MIN;
        let mut max_blocks = usize::MIN;
        let mut start_addresses = HashSet::new();
        let mut end_addresses = HashSet::new();

        let mut blocks = 0;

        let mut current_map_snapshot = NoHashMap::default();
        for (counter, instruction) in instructions.iter().enumerate() {
            let operation = instruction.get_operation();
            let instruction_string = operation.to_string();
            match operation {
                MemoryUpdate::Allocation(..) =>
                    eprintln!("Processing instruction {}: {}", counter.cyan(), instruction_string.red()),
                MemoryUpdate::Free(..) =>
                    eprintln!("Processing instruction {}: {}", counter.cyan(), instruction_string.green()),
            }
            if counter % MAP_CACHE_SIZE == 0 {
                eprintln!("Caching map...");
                self.memory_map_snapshots.push(current_map_snapshot.clone());
            }

            let modified_blocks = self.count_modified_bytes(&instruction.get_operation(), &self.operation_history_map);
            let mut usage = self.calculate_memory_usage(&instruction, modified_blocks);
            Self::count_blocks_in_memory(&instruction.get_operation(), &mut start_addresses, &mut end_addresses, &mut blocks);
            usage.blocks = blocks;

            max_usage = max(usage.memory_used_absolute, max_usage);
            max_blocks = max(usage.blocks, max_blocks);

            self.memory_usage_snapshots.push(usage);
            match instruction.get_operation() {
                MemoryUpdate::Allocation(address, size, callstack) =>
                    map_manipulator::allocate_memory(&mut current_map_snapshot, address, size, callstack),
                MemoryUpdate::Free(address, callstack) => {
                    map_manipulator::free_memory(&mut current_map_snapshot, address, callstack);
                }
            }
            self.log_operation(instruction.clone());
        }

        self.max_usage = max_usage;
        self.max_blocks = max_blocks;
    }

    pub fn calculate_memory_usage(&self, instruction: &Instruction, modified_blocks: usize) -> MemoryUsage {
        unsafe {
            // calculate total usage
            static mut MEMORY_USED_ABSOLUTE: usize = 0;
            let latest_address = match instruction.get_operation() {
                MemoryUpdate::Allocation(address, ..) => {
                    MEMORY_USED_ABSOLUTE += modified_blocks;
                    address
                },
                MemoryUpdate::Free(address, ..) => {
                    MEMORY_USED_ABSOLUTE -= modified_blocks;
                    address
                },
            };

            MemoryUsage {
                memory_used_absolute: MEMORY_USED_ABSOLUTE,
                total_memory: consts::DEFAULT_MEMORY_SIZE,
                blocks: 0,
                latest_operation: latest_address,
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
        self.instructions.push(instruction);
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
                    latest_operation: 0,
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
            latest_operation: 0,
        }
    }

    pub fn get_memory_usage_view(&self, span_start: usize, span_end: usize) -> Vec<[f64; 2]> {
        let mut vector = Vec::new();
        for i in span_start..min(self.memory_usage_snapshots.len(), span_end) {
            let memory_used_absolute = self.memory_usage_snapshots.get(i)
                .expect("[DamselflyViewer::get_memory_usage_view]: Error getting timestamp from memory_usage_snapshots")
                .memory_used_absolute;
            let memory_used_percentage = memory_used_absolute as f64 * 100.0 / self.max_usage as f64;
            vector.push([(i - span_start) as f64, memory_used_percentage]);
        }
        vector
    }

    pub fn get_latest_map_state(&self) -> (NoHashMap<usize, MemoryStatus>, Option<&MemoryUpdate>) {
        (self.memory_map.clone(), self.operation_history.get(self.get_total_operations().saturating_sub(1)))
    }

    pub fn get_map_state(&self, time: usize, absolute_span_start: usize, absolute_span_end: usize) -> (NoHashMap<usize, MemoryStatus>, Option<&MemoryUpdate>) {
        let starting_snapshot = time / MAP_CACHE_SIZE;
        let mut map =
            Self::clone_map_partial(self.memory_map_snapshots.get(starting_snapshot).unwrap(),
                                    map_manipulator::absolute_to_logical(absolute_span_start),
                                    map_manipulator::absolute_to_logical(absolute_span_end));

        for cur_time in starting_snapshot * MAP_CACHE_SIZE..=time {
            if let Some(operation) = self.operation_history.get(cur_time) {
                match operation {
                    MemoryUpdate::Allocation(absolute_address, size, callstack) => {
                        if let Some((fill_start, bytes_to_fill)) = Self::bytes_allocated_within_span(*absolute_address, *size, absolute_span_start, absolute_span_end) {
                            Self::allocate_memory(&mut map, fill_start, bytes_to_fill, Rc::clone(callstack));
                        }
                    }
                    MemoryUpdate::Free(absolute_address, callstack) => {
                        if let Some((free_start, bytes_to_free)) = Self::bytes_freed_within_span(*absolute_address, absolute_span_start, absolute_span_end, &map) {
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

    pub fn bytes_allocated_within_span(op_address: usize, size: usize, absolute_span_start: usize, absolute_span_end: usize) -> Option<(usize, usize)> {
        let span_size = (absolute_span_end - absolute_span_start) * DEFAULT_BLOCK_SIZE;
        // allocation starts before span and does not continue into it
        if op_address + size < absolute_span_start {
            return None;
        }
        // allocation starts before span, but continues into it
        if op_address < absolute_span_start {
            let bytes_to_fill = min(span_size, op_address + size - absolute_span_start);
            return Some((absolute_span_start, bytes_to_fill));
        }
        // allocation starts inside span
        if op_address >= absolute_span_start && op_address <= absolute_span_end {
            let bytes_to_fill = min(size, span_size);
            return Some((op_address, bytes_to_fill));
        }
        // allocation starts after span
        None
    }

    pub fn bytes_freed_within_span(parent_block: usize, absolute_span_start: usize, absolute_span_end: usize, map: &NoHashMap<usize, MemoryStatus>) -> Option<(usize, usize)> {
        let start = max(parent_block, absolute_span_start);
        let mut current_address = start;
        // If free happens after the span, it need not be reflected within the span
        if parent_block > absolute_span_end {
            return None;
        }

        while let Some(block_status) = map_manipulator::view_memory(map, current_address) {
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
            if current_address > absolute_span_end {
                return Some((start, current_address - start));
            }
        }
        Some((start, current_address - start))
    }

    pub fn count_adjacent_allocated_blocks(start_address: usize, end_address: usize, parent_block_address: usize, map: &NoHashMap<usize, MemoryStatus>) -> usize {
        let mut current_address = start_address;
        current_address += 1;
        while let Some(next_block) = map_manipulator::view_memory(map, current_address) {
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

    fn free_memory_manual(map: &mut NoHashMap<usize, MemoryStatus>, absolute_address: usize, bytes: usize, callstack: Rc<String>) {
        let scaled_address = map_manipulator::absolute_to_logical(absolute_address);
        if scaled_address * DEFAULT_BLOCK_SIZE != absolute_address {
            panic!("[DamselflyViewer::free_memory]: Block arithmetic error");
        }
        let blocks = bytes / DEFAULT_BLOCK_SIZE;
        for i in 0..blocks {
            map.insert(scaled_address + i, MemoryStatus::Free(Rc::clone(&callstack)));
        }
    }

    pub fn allocate_memory(map: &mut NoHashMap<usize, MemoryStatus>, absolute_address: usize, bytes: usize, callstack: Rc<String>) {
        let scaled_address = map_manipulator::absolute_to_logical(absolute_address);
        if bytes == 0 {
            return;
        }
        if bytes < DEFAULT_BLOCK_SIZE && bytes > 0 {
            map.insert(scaled_address, MemoryStatus::PartiallyAllocated(absolute_address, callstack));
            return;
        }

        let full_blocks = map_manipulator::absolute_to_logical(bytes);
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

    pub fn get_total_operations(&self) -> usize {
        self.memory_usage_snapshots.len()
    }

    pub fn get_operation_log(&self, start: usize, end: usize) -> &[MemoryUpdate] {
        &self.operation_history[start..=end]
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
}