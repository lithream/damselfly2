use std::iter::StepBy;
use std::ops::Range;
use rust_lapper::Lapper;
use crate::damselfly2::consts::DEFAULT_MEMORYSPAN;
use crate::damselfly2::memory::memory_status::MemoryStatus;
use crate::damselfly2::memory::memory_update::{MemoryUpdate, MemoryUpdateType};
use crate::damselfly2::update_interval::UpdateInterval;

pub struct Bucket {
    free_span: (usize, usize),
    bucket_size: usize,
    bucket_status: MemoryStatus,
}

impl Bucket {
    pub fn new(bucket_index: usize, bucket_size: usize) -> Bucket {
        Bucket {
            free_span: (bucket_index, bucket_index + bucket_size),
            bucket_size,
            bucket_status: MemoryStatus::Unused,
        }
    }

    pub fn update_block(&mut self, update_interval: UpdateInterval) {
        if update_interval.start <= self.free_span.0 && update_interval.stop >= self.free_span.1 {
            self.apply_update(update_interval.val);
            return;
        }
        self.apply_partial_update(update_interval.start, update_interval.stop, update_interval.val);
    }

    fn apply_update(&mut self, memory_update: MemoryUpdateType) {
        match memory_update {
            MemoryUpdateType::Allocation(allocation) => self.bucket_status
                = MemoryStatus::Allocated(allocation.get_absolute_address(),
                                          allocation.get_absolute_size(),
                                          allocation.get_callstack()),
            MemoryUpdateType::Free(free) => self.bucket_status
                = MemoryStatus::Free(free.get_callstack()),
        }
    }

    fn apply_partial_update(&mut self, update_start: usize, update_stop: usize, update: MemoryUpdateType) {
        if matches!(self.bucket_status, MemoryStatus::PartiallyAllocated(..)) {
            let current_state_size = self.bucket_
        }
    }
}

pub struct MemoryCanvas {
    block_size: usize,
    start: usize,
    stop: usize,
}

impl MemoryCanvas {
    pub fn new() -> MemoryCanvas {
        MemoryCanvas {
            block_size: 1,
            start: 0,
            stop: DEFAULT_MEMORYSPAN,
        }
    }

    pub fn load_intervals(&self, update_intervals: Vec<UpdateInterval>) {
        let lapper = Lapper::new(update_intervals);
        let buckets = Vec::new();
        let visible_blocks = self.get_block_iter();
        for block_start in visible_blocks {
            let overlapping_operations
                = lapper.find(block_start, block_start + self.block_size);

        }
    }

    fn get_block_iter(&self) -> StepBy<Range<usize>> {
        (self.start..self.stop).step_by(self.block_size)
    }
}