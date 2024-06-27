//! A canvas of memory, used to draw the memory map.
use std::iter::StepBy;
use std::ops::Range;
use rust_lapper::Lapper;
use crate::damselfly::memory::memory_status::MemoryStatus;
use crate::damselfly::memory::memory_update::MemoryUpdateType;
use crate::damselfly::update_interval::update_interval_sorter::UpdateIntervalSorter;
use crate::damselfly::update_interval::UpdateInterval;
use crate::damselfly::viewer::memory_block::Block;

#[derive(Clone)]
pub struct MemoryCanvas {
    block_size: usize,
    start: usize,
    stop: usize,
    pub blocks: Vec<Block>,
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

    /// Paints the canvas based on the blocks it has stored.
    pub fn paint_blocks(&mut self) {
        self.insert_blocks();
        for block in &mut self.blocks {
            let mut overlapping_operations
                = self.full_lapper.find(block.get_block_start(), block.get_block_stop())
                .collect::<Vec<&UpdateInterval>>();
            UpdateIntervalSorter::sort_by_timestamp(&mut overlapping_operations);

            for update in overlapping_operations.iter() {
                block.paint_block(&update.val);
            }
        }
    }

    /// Paints the existing canvas with list of temporary updates.
    /// You might want to call paint_blocks first to paint the canvas with its own blocks, then
    /// call this to paint over it with the new updates.
    /// 
    /// # Arguments 
    /// 
    /// * `temporary_updates`: Updates to paint over the canvas.
    pub fn paint_temporary_updates(&mut self, temporary_updates: Vec<UpdateInterval>) {
        let temp_lapper = Lapper::new(temporary_updates);
        for block in &mut self.blocks {
            let mut overlapping_operations 
                = temp_lapper.find(block.get_block_start(), block.get_block_stop())
                        .collect::<Vec<&UpdateInterval>>();
            UpdateIntervalSorter::sort_by_timestamp(&mut overlapping_operations);

            for update in overlapping_operations.iter() {
                block.paint_block(&update.val);
            }
        }
    }
    
    /// Paints temporary updates onto the current canvas, but does not modify the canvas. Instead,
    /// it makes a copy and returns it.
    /// 
    /// # Arguments 
    /// 
    /// * `temporary_updates`: Updates to paint over the canvas.
    /// 
    /// returns: Vec<Block, Global> 
    pub fn simulate_painting_temporary_updates(&self, temporary_updates: Vec<UpdateInterval>) -> Vec<Block> {
        let temp_lapper = Lapper::new(temporary_updates);
        let mut blocks = self.blocks.clone();
        for block in &mut blocks {
            let mut overlapping_operations
                = temp_lapper.find(block.get_block_start(), block.get_block_stop())
                        .collect::<Vec<&UpdateInterval>>();
            UpdateIntervalSorter::sort_by_timestamp(&mut overlapping_operations);

            for update in overlapping_operations.iter() {
                block.paint_block(&update.val);
            }
        }
        blocks
    }

    /// Paints the map and returns a Vec of MemoryStatus representing the map.
    pub fn render(&mut self) -> Vec<MemoryStatus> {
        self.paint_blocks();
        self.blocks
            .iter()
            .map(|block| block.block_status.clone())
            .collect()
    }

    /// Simulates painting the map with temporary updates and returns a Vec of MemoryStatus representing
    /// the map.
    pub fn render_temporary(&self, temporary_updates: Vec<UpdateInterval>) -> Vec<MemoryStatus> {
        let blocks = self.simulate_painting_temporary_updates(temporary_updates);
        let mut block_statuses = Vec::new();
        for block in blocks {
            block_statuses.push(block.block_status);
        }
        block_statuses
    }

    pub fn insert_blocks(&mut self) {
        for block_address in self.get_block_iter() {
            self.blocks.push(Block::new(block_address, self.block_size));
        }
    }

    fn get_block_iter(&self) -> StepBy<Range<usize>> {
        (self.start..self.stop).step_by(self.block_size)
    }
}

#[cfg(test)]
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
        assert!(matches!(canvas_status[1], MemoryStatus::Unused(..)));
        assert!(matches!(canvas_status[2], MemoryStatus::Allocated(..)));
        assert!(matches!(canvas_status[3], MemoryStatus::Allocated(..)));
        assert!(matches!(canvas_status[4], MemoryStatus::Unused(..)));
        assert!(matches!(canvas_status[5], MemoryStatus::PartiallyAllocated(..)));
        assert!(matches!(canvas_status[6], MemoryStatus::PartiallyAllocated(..)));
        assert!(matches!(canvas_status[7], MemoryStatus::Unused(..)));
        assert!(matches!(canvas_status[8], MemoryStatus::Unused(..)));
    }
}