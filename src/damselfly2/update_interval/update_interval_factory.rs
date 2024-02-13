use std::sync::Arc;
use rust_lapper::{Interval, Lapper};
use crate::damselfly2::memory_update::MemoryUpdate;
use crate::damselfly2::memory_structs::NoHashMap;
use crate::damselfly2::memory_update::MemoryUpdateType;


pub struct UpdateIntervalFactory {
    memory_updates: Vec<Box<dyn MemoryUpdate>>,
    lapper: Lapper<usize, MemoryUpdateType>,
}

impl UpdateIntervalFactory {
    pub fn load_instruction<T: MemoryUpdate>(&mut self, update: T)
    {
        self.memory_updates.push(Box::new(update));
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
}