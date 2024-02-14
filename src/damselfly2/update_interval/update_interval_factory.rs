use rust_lapper::{Interval, Lapper};
use crate::damselfly2::memory::memory_update::{MemoryUpdate, MemoryUpdateType};
use crate::damselfly2::update_interval::UpdateInterval;


pub struct UpdateIntervalFactory {
    memory_updates: Vec<MemoryUpdateType>,
    lapper: Lapper<usize, MemoryUpdateType>,
}

impl UpdateIntervalFactory {
    pub fn append_instruction(&mut self, update: MemoryUpdateType)
    {
        self.memory_updates.push(update);
    }
    pub fn load_instructions(&mut self, updates: Vec<MemoryUpdateType>) {
        self.memory_updates = updates;
    }

    pub fn calculate_overlaps(&mut self) {
        let intervals = self.construct_enum_vector();
        self.lapper = Lapper::new(intervals);
    }

    // start..end, not start..=end
    pub fn find_overlaps(&self, start: usize, end: usize) -> Vec<&UpdateInterval> {
        self.lapper.find(start, end).collect::<Vec<&UpdateInterval>>()
    }

    fn construct_enum_vector(&self) -> Vec<Interval<usize, MemoryUpdateType>> {
        let mut intervals = Vec::new();
        for memory_update in self.memory_updates {
            let start = memory_update.get_absolute_address();
            let stop = start + memory_update.get_absolute_size();
            intervals.push(UpdateInterval{ start, stop, val: memory_update.wrap_in_enum() });
        }

        intervals
    }

    fn get_start_and_stop(&self, memory_update: &MemoryUpdateType) -> (usize, usize) {
        match memory_update {
            MemoryUpdateType::Allocation(allocation) => {
                let alloc_address = allocation.get_absolute_address();
                (alloc_address, alloc_address + allocation.get_absolute_size())
            }
            MemoryUpdateType::Free(free) => {
                let free_address = free.get_absolute_address();
                (free_address, free_address + free.get_absolute_size())
            }
        }
    }
}