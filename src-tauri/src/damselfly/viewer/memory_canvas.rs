use std::cmp::{max, min};
use std::iter::StepBy;
use std::ops::Range;
use std::sync::Arc;
use rust_lapper::Lapper;
use crate::damselfly::memory::memory_status::MemoryStatus;
use crate::damselfly::memory::memory_update::{MemoryUpdate, MemoryUpdateType};
use crate::damselfly::update_interval::update_interval_sorter::UpdateIntervalSorter;
use crate::damselfly::update_interval::update_queue_compressor::UpdateQueueCompressor;
use crate::damselfly::update_interval::UpdateInterval;

#[derive(Clone)]
pub struct Block {
    block_bounds: (usize, usize),
    remaining_bytes: usize,
    block_status: MemoryStatus,
}

impl Block {
    pub fn new(block_index: usize, block_size: usize) -> Block {
        Block {
            block_bounds: (block_index, block_index + block_size),
            remaining_bytes: block_size,
            block_status: MemoryStatus::Unused,
        }
    }

    pub fn paint_block(&mut self, update_interval: &MemoryUpdateType) {
        let constrained_bounds = (
            max(self.block_bounds.0, update_interval.get_start()),
            min(self.block_bounds.1, update_interval.get_end())
        );
        let bytes_consumed = constrained_bounds.1 - constrained_bounds.0;
        match &update_interval {
            MemoryUpdateType::Allocation(allocation) => {
                self.remaining_bytes = self.remaining_bytes.saturating_sub(bytes_consumed);
                self.update_block_status(allocation.get_absolute_address(), allocation.get_absolute_size(), allocation.get_callstack());
            }
            MemoryUpdateType::Free(free) => {
                self.remaining_bytes = self.remaining_bytes.saturating_add(bytes_consumed)
                    .clamp(usize::MIN, self.block_bounds.1 - self.block_bounds.0);
                self.update_block_status(free.get_absolute_address(), free.get_absolute_size(), free.get_callstack());
            }
        }
    }

    pub fn get_block_status(&self) -> &MemoryStatus {
        &self.block_status
    }

    pub fn get_block_start(&self) -> usize {
        self.block_bounds.0
    }

    pub fn get_block_stop(&self) -> usize {
        self.block_bounds.1
    }

    fn update_block_status(&mut self, absolute_address: usize, absolute_size: usize, callstack: Arc<String>) {
        if self.remaining_bytes == 0 {
            self.block_status = MemoryStatus::Allocated(absolute_address, absolute_size, callstack);
        } else if self.remaining_bytes < (self.block_bounds.1 - self.block_bounds.0) {
            self.block_status = MemoryStatus::PartiallyAllocated(absolute_address, absolute_size, callstack);
        } else if self.remaining_bytes == (self.block_bounds.1 - self.block_bounds.0) {
            self.block_status = MemoryStatus::Free(absolute_address, absolute_size, callstack);
        } else {
            panic!("[MemoryCanvas::update_block_status]: Remaining bytes exceeds span");
        }
    }
}

#[derive(Clone)]
pub struct MemoryCanvas {
    block_size: usize,
    start: usize,
    stop: usize,
    blocks: Vec<Block>,
    full_lapper: Lapper<usize, MemoryUpdateType>,
}

impl MemoryCanvas {
    pub fn new(start: usize, stop: usize, block_size: usize, update_intervals: Vec<UpdateInterval>) -> MemoryCanvas {
        MemoryCanvas {
            block_size,
            start,
            stop,
            blocks: Vec::new(),
            full_lapper: Lapper::new(update_intervals),
        }
    }

    pub fn paint_blocks(&mut self) {
        self.insert_blocks();
        for block in &mut self.blocks {
            let mut overlapping_operations
                = self.full_lapper.find(block.get_block_start(), block.get_block_stop())
                        .collect::<Vec<&UpdateInterval>>();
            UpdateIntervalSorter::sort_by_timestamp(&mut overlapping_operations);
            let compressed_intervals = UpdateQueueCompressor::compress_intervals(overlapping_operations);
            let mut interval_iter = compressed_intervals.iter().rev();
            loop {
                if let MemoryStatus::Allocated(_, _, _) = block.get_block_status() { break }
                if let Some(update_interval) = interval_iter.next() {
                    block.paint_block(update_interval);
                } else {
                    break;
                }
            }
        }
    }

    pub fn render(&mut self) -> Vec<MemoryStatus> {
        self.paint_blocks();
        self.blocks
            .iter()
            .map(|block| block.block_status.clone())
            .collect()
    }

    pub fn set_start(&mut self, new_start: usize) {
        self.start = new_start;
    }

    pub fn add_to_start(&mut self, start_delta: usize) {
        self.start = self.start.saturating_add(start_delta);
    }

    pub fn sub_from_start(&mut self, start_delta: usize) {
        self.start = self.start.saturating_sub(start_delta);
    }

    pub fn set_stop(&mut self, new_stop: usize) {
        self.stop = new_stop;
    }

    pub fn add_to_stop(&mut self, stop_delta: usize) {
        self.stop = self.stop.saturating_add(stop_delta);
    }

    pub fn sub_from_stop(&mut self, stop_delta: usize) {
        self.stop = self.stop.saturating_sub(stop_delta);
    }

    fn insert_blocks(&mut self) {
        for block_address in self.get_block_iter() {
            self.blocks.push(Block::new(block_address, self.block_size));
        }
    }

    fn get_block_iter(&self) -> StepBy<Range<usize>> {
        (self.start..self.stop).step_by(self.block_size)
    }

    fn get_intervals_overlapping_window(&self) -> Vec<UpdateInterval> {
        let window_overlaps = self.full_lapper.find(self.start, self.stop).collect::<Vec<&UpdateInterval>>();
        let mut res = Vec::new();
        for overlap in window_overlaps {
            res.push(overlap.clone());
        }
        res
    }
}
