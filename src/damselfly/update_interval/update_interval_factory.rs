use rust_lapper::{Interval};
use crate::damselfly::memory::memory_update::MemoryUpdateType;
use crate::damselfly::update_interval::UpdateInterval;
use crate::damselfly::update_interval::utility::Utility;


pub struct UpdateIntervalFactory {
    memory_updates: Vec<MemoryUpdateType>,
}

impl UpdateIntervalFactory {
    pub fn new(memory_updates: Vec<MemoryUpdateType>) -> UpdateIntervalFactory {
        UpdateIntervalFactory {
            memory_updates,
        }
    }
    pub fn append_update(&mut self, update: MemoryUpdateType)
    {
        self.memory_updates.push(update);
    }

    pub fn load_instructions(&mut self, updates: Vec<MemoryUpdateType>) {
        self.memory_updates = updates;
    }

    pub fn construct_enum_vector(&self) -> Vec<Interval<usize, MemoryUpdateType>> {
        let mut intervals = Vec::new();
        for memory_update in &self.memory_updates {
            let (start, stop) = Utility::get_start_and_stop(memory_update);
            intervals.push(UpdateInterval { start, stop, val: memory_update.clone() });
        }

        intervals
    }

    pub fn convert_update_to_interval(memory_update: &MemoryUpdateType) -> UpdateInterval {
        let (start, stop) = Utility::get_start_and_stop(memory_update);
        UpdateInterval{ start, stop, val: memory_update.clone() }
    }
}