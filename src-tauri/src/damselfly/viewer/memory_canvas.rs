use std::cmp::{max, min};
use std::collections::HashSet;
use std::iter::StepBy;
use std::ops::Range;
use std::sync::Arc;
use rust_lapper::Lapper;
use crate::damselfly::memory::memory_status::MemoryStatus;
use crate::damselfly::memory::memory_update::{MemoryUpdate, MemoryUpdateType};
use crate::damselfly::update_interval::update_interval_sorter::UpdateIntervalSorter;
use crate::damselfly::update_interval::update_queue_compressor::UpdateQueueCompressor;
use crate::damselfly::update_interval::UpdateInterval;
use crate::damselfly::viewer::memory_block::Block;


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
            let mut update_blacklist = HashSet::new();
//            let compressed_intervals = UpdateQueueCompressor::compress_intervals(overlapping_operations);
            let mut interval_iter = overlapping_operations.iter().rev();
            loop {
                if let MemoryStatus::Allocated(_, _, _) = block.get_block_status() { break }
                if let Some(update_interval) = interval_iter.next() {
                    match &update_interval.val {
                        MemoryUpdateType::Allocation(allocation) => {
                            if !update_blacklist.contains(&allocation.get_absolute_address()) {
                                block.paint_block(&update_interval.val);
                            }
                        }
                        MemoryUpdateType::Free(free) => {
                            update_blacklist.insert(free.get_absolute_address());
                        }
                    }
                } else {
                    break;
                }
            }
        }
    }
    
    pub fn paint_over_blocks(&self, temporary_updates: Vec<UpdateInterval>) -> Vec<Block> {
        let temp_lapper = Lapper::new(temporary_updates);
        let mut blocks = self.blocks.clone();
        for block in &mut blocks {
            let mut overlapping_operations
                = temp_lapper.find(block.get_block_start(), block.get_block_stop())
                        .collect::<Vec<&UpdateInterval>>();
            UpdateIntervalSorter::sort_by_timestamp(&mut overlapping_operations);
            let mut update_blacklist = HashSet::new();
//            let compressed_intervals = UpdateQueueCompressor::compress_intervals(overlapping_operations);
            let mut interval_iter = overlapping_operations.iter().rev();
            loop {
                if let MemoryStatus::Allocated(_, _, _) = block.get_block_status() { break }
                if let Some(update_interval) = interval_iter.next() {
                    match &update_interval.val {
                        MemoryUpdateType::Allocation(allocation) => {
                            if !update_blacklist.contains(&allocation.get_absolute_address()) {
                                block.paint_block(&update_interval.val);
                            }
                        }
                        MemoryUpdateType::Free(free) => {
                            update_blacklist.insert(free.get_absolute_address());
                        }
                    }
                } else {
                    break;
                }
            }
        }
        blocks
    }

    pub fn render(&mut self) -> Vec<MemoryStatus> {
        self.paint_blocks();
        self.blocks
            .iter()
            .map(|block| block.block_status.clone())
            .collect()
    }
    
    pub fn render_temporary(&self, temporary_updates: Vec<UpdateInterval>) -> Vec<MemoryStatus> {
        let blocks = self.paint_over_blocks(temporary_updates);
        let mut block_statuses = Vec::new();
        for block in blocks {
            block_statuses.push(block.block_status);
        }
        block_statuses
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

mod tests {
    use std::sync::Arc;
    use crate::damselfly::memory::memory_status::MemoryStatus;
    use crate::damselfly::memory::memory_update::{Allocation, MemoryUpdateType};
    use crate::damselfly::update_interval::update_interval_factory::UpdateIntervalFactory;
    use crate::damselfly::viewer::memory_canvas::MemoryCanvas;

    #[test]
    fn only_allocs() {
        let updates = vec![
            MemoryUpdateType::Allocation(Allocation::new(0, 4, Arc::new("test".to_string()), 0, "0".to_string())),
            MemoryUpdateType::Allocation(Allocation::new(8, 4, Arc::new("test".to_string()), 0, "0".to_string())),
            MemoryUpdateType::Allocation(Allocation::new(12, 4, Arc::new("test".to_string()), 0, "0".to_string())),
            MemoryUpdateType::Allocation(Allocation::new(22, 4, Arc::new("test".to_string()), 0, "0".to_string())),
        ];
        let update_intervals = UpdateIntervalFactory::new(updates).construct_enum_vector();
        let mut canvas = MemoryCanvas::new(0, 128, 4, update_intervals);
        let canvas_status = canvas.render();
        assert!(matches!(canvas_status[0], MemoryStatus::Allocated(..)));
        assert!(matches!(canvas_status[1], MemoryStatus::Unused));
        assert!(matches!(canvas_status[2], MemoryStatus::Allocated(..)));
        assert!(matches!(canvas_status[3], MemoryStatus::Allocated(..)));
        assert!(matches!(canvas_status[4], MemoryStatus::Unused));
        assert!(matches!(canvas_status[5], MemoryStatus::PartiallyAllocated(..)));
        assert!(matches!(canvas_status[6], MemoryStatus::PartiallyAllocated(..)));
        assert!(matches!(canvas_status[7], MemoryStatus::Unused));
        assert!(matches!(canvas_status[8], MemoryStatus::Unused));
    }
}