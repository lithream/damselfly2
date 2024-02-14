use rust_lapper::{Interval, Lapper};
use crate::damselfly2::memory::memory_update::{MemoryUpdate, MemoryUpdateType};
use crate::damselfly2::update_interval::UpdateInterval;


pub struct UpdateIntervalFactory {
    memory_updates: Vec<MemoryUpdateType>,
}

impl UpdateIntervalFactory {
    pub fn append_instruction(&mut self, update: MemoryUpdateType)
    {
        self.memory_updates.push(update);
    }
    pub fn load_instructions(&mut self, updates: Vec<MemoryUpdateType>) {
        self.memory_updates = updates;
    }



    fn construct_enum_vector(&self) -> Vec<Interval<usize, MemoryUpdateType>> {
        let mut intervals = Vec::new();
        for memory_update in self.memory_updates {
            let (start, stop) = self.get_start_and_stop(&memory_update);
            intervals.push(UpdateInterval { start, stop, val: memory_update });
        }

        intervals
    }

    fn get_start_and_stop(&self, memory_update: &MemoryUpdateType) -> (usize, usize) {
        let (start, stop) = match memory_update {
            MemoryUpdateType::Allocation(allocation) => {
                let alloc_address = allocation.get_absolute_address();
                (alloc_address, alloc_address + allocation.get_absolute_size())
            }
            MemoryUpdateType::Free(free) => {
                let free_address = free.get_absolute_address();
                (free_address, free_address + free.get_absolute_size())
            }
        };
        
        (start, stop)
    }
}