use std::cmp::{max, min};
use std::iter::StepBy;
use std::ops::Range;
use rust_lapper::Lapper;
use crate::damselfly2::consts::DEFAULT_MEMORYSPAN;
use crate::damselfly2::memory::memory_status::MemoryStatus;
use crate::damselfly2::memory::memory_update::MemoryUpdateType;
use crate::damselfly2::update_interval::update_interval_sorter::UpdateIntervalSorter;
use crate::damselfly2::update_interval::UpdateInterval;

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

    pub fn update_block(&mut self, update_interval: &UpdateInterval) {
        let constrained_bounds = (
            max(self.block_bounds.0, update_interval.start),
            min(self.block_bounds.1, update_interval.stop)
        );
        let bytes_consumed = constrained_bounds.1 - constrained_bounds.0;
        match update_interval.val {
            MemoryUpdateType::Allocation(_) => {
                self.remaining_bytes = self.remaining_bytes.saturating_sub(bytes_consumed);
            }
            MemoryUpdateType::Free(_) => {
                self.remaining_bytes = self.remaining_bytes.saturating_add(bytes_consumed)
                    .clamp(usize::MIN, self.block_bounds.1 - self.block_bounds.0);
            }
        }
        self.remaining_bytes = self.remaining_bytes.saturating_sub(bytes_consumed);
        self.update_block_status(update_interval.clone().val);
    }

    pub fn get_block_start(&self) -> usize {
        self.block_bounds.0
    }

    pub fn get_block_stop(&self) -> usize {
        self.block_bounds.1
    }

    fn update_block_status(&mut self, memory_update: MemoryUpdateType) {
        if self.remaining_bytes == 0 {
            self.block_status = MemoryStatus::Free;
        } else if self.remaining_bytes < (self.block_bounds.1 - self.block_bounds.0) {
            self.block_status = MemoryStatus::PartiallyAllocated;
        } else if self.remaining_bytes >= (self.block_bounds.1 - self.block_bounds.0) {
            self.block_status = MemoryStatus::Allocated;
        }
    }
}

pub struct MemoryCanvas {
    block_size: usize,
    start: usize,
    stop: usize,
    blocks: Vec<Block>
}

impl MemoryCanvas {
    pub fn new() -> MemoryCanvas {
        MemoryCanvas {
            block_size: 1,
            start: 0,
            stop: DEFAULT_MEMORYSPAN,
            blocks: Vec::new(),
        }
    }

    pub fn load_intervals(&mut self, update_intervals: Vec<UpdateInterval>) {
        let lapper = Lapper::new(update_intervals);
        self.insert_blocks();
        for mut block in self.blocks {
            let mut overlapping_operations
                = lapper.find(block.get_block_start(), block.get_block_stop())
                        .collect::<Vec<&UpdateInterval>>();
            UpdateIntervalSorter::sort_by_timestamp(&mut overlapping_operations);
            for update_interval in overlapping_operations {
                block.update_block(update_interval);
            }
        }
    }

    fn insert_blocks(&mut self) {
        for block in self.get_block_iter() {
            let address = self.start + block;
            self.blocks.push(Block::new(address, self.block_size));
        }
    }

    fn get_block_iter(&self) -> StepBy<Range<usize>> {
        (self.start..self.stop).step_by(self.block_size)
    }
}