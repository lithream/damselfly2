use std::cmp::{max, min};
use std::sync::Arc;
use crate::damselfly::memory::memory_status::MemoryStatus;
use crate::damselfly::memory::memory_update::{MemoryUpdate, MemoryUpdateType};

#[derive(Clone)]
pub struct Block {
    block_bounds: (usize, usize),
    remaining_bytes: usize,
    pub block_status: MemoryStatus,
}

impl Block {
    pub fn new(block_index: usize, block_size: usize) -> Block {
        Block {
            block_bounds: (block_index, block_index + block_size),
            remaining_bytes: block_size,
            block_status: MemoryStatus::Unused(block_index),
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
            self.block_status = MemoryStatus::Allocated(absolute_address, absolute_size, self.block_status.get_address(), callstack);
        } else if self.remaining_bytes < (self.block_bounds.1 - self.block_bounds.0) {
            self.block_status = MemoryStatus::PartiallyAllocated(absolute_address, absolute_size, self.block_status.get_address(), callstack);
        } else if self.remaining_bytes == (self.block_bounds.1 - self.block_bounds.0) {
            self.block_status = MemoryStatus::Free(absolute_address, absolute_size, self.block_status.get_address(), callstack);
        } else {
            panic!("[MemoryCanvas::update_block_status]: Remaining bytes exceeds span");
        }
    }
}

mod tests {
    use std::sync::Arc;
    use crate::damselfly::memory::memory_status::MemoryStatus;
    use crate::damselfly::memory::memory_update::{Allocation, Free, MemoryUpdateType};
    use crate::damselfly::viewer::memory_block::Block;

    #[test]
    fn paint_block_one_alloc_exact_overlap() {
        let mut block = Block::new(0, 4);
        block.paint_block(&MemoryUpdateType::Allocation(Allocation::new(0, 4, Arc::new("test".to_string()), 0, "0".to_string())));
        assert!(matches!(*block.get_block_status(), MemoryStatus::Allocated(..)));
    }

    #[test]
    fn paint_block_one_alloc_partial_overlap_left() {
        let mut block = Block::new(4, 4);
        block.paint_block(&MemoryUpdateType::Allocation(Allocation::new(0, 6, Arc::new("left".to_string()), 0, "0".to_string())));
        assert!(matches!(*block.get_block_status(), MemoryStatus::PartiallyAllocated(..)));
        assert_eq!(block.remaining_bytes, 2);
    }

    #[test]
    fn paint_block_one_alloc_partial_overlap_right() {
        let mut block = Block::new(4, 4);
        block.paint_block(&MemoryUpdateType::Allocation(Allocation::new(6, 4, Arc::new("right".to_string()), 0, "0".to_string())));
        assert!(matches!(*block.get_block_status(), MemoryStatus::PartiallyAllocated(..)));
        assert_eq!(block.remaining_bytes, 2);
    }

    #[test]
    fn paint_block_two_allocs() {
        let mut block = Block::new(4, 4);
        let allocs = vec![
            MemoryUpdateType::Allocation(Allocation::new(0, 6, Arc::new("left".to_string()), 0, "0".to_string())),
            MemoryUpdateType::Allocation(Allocation::new(6, 4, Arc::new("right".to_string()), 1, "1".to_string())),
        ];
        for alloc in &allocs {
            block.paint_block(alloc);
        }
        assert!(matches!(block.get_block_status(), MemoryStatus::Allocated(..)));
        assert_eq!(block.remaining_bytes, 0);
    }

    #[test]
    fn paint_block_allocs_and_frees() {
        let mut block = Block::new(4, 4);
        let ops = vec![
            MemoryUpdateType::Allocation(Allocation::new(0, 6, Arc::new("left".to_string()), 0, "0".to_string())),
            MemoryUpdateType::Free(Free::new(0, 6, Arc::new("free".to_string()), 1, "1".to_string())),
        ];
        for op in &ops {
            block.paint_block(op);
        }
        assert!(matches!(block.get_block_status(), MemoryStatus::Free(..)));
        assert_eq!(block.remaining_bytes, 4);
    }
}