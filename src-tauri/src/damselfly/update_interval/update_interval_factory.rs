//! Converts memory updates into UpdateInterval objects which can be used with lapper trees.
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
    /// Appends an update to the current queue of updates.
    /// 
    /// # Arguments 
    /// 
    /// * `update`: Update to append.
    /// 
    /// returns: () 
    pub fn append_update(&mut self, update: MemoryUpdateType)
    {
        self.memory_updates.push(update);
    }

    /// Replaces updates.
    /// 
    /// # Arguments 
    /// 
    /// * `updates`: Updates to replace with.
    /// 
    /// returns: () 
    pub fn load_updates(&mut self, updates: Vec<MemoryUpdateType>) {
        self.memory_updates = updates;
    }

    /// Constructs a vector of memory updates as Intervals for use with lapper trees.
    /// 
    /// returns: Vec<Interval<usize, MemoryUpdateType>>
    pub fn construct_enum_vector(&self) -> Vec<Interval<usize, MemoryUpdateType>> {
        let mut intervals = Vec::new();
        for memory_update in &self.memory_updates {
            let (start, stop) = Utility::get_start_and_stop(memory_update);
            intervals.push(UpdateInterval { start, stop, val: memory_update.clone() });
        }

        intervals
    }

    /// Converts a MemoryUpdate to an UpdateInterval
    /// 
    /// # Arguments 
    /// 
    /// * `memory_update`: The update to convert.
    /// 
    /// returns: Interval<usize, MemoryUpdateType> 
    pub fn convert_update_to_interval(memory_update: &MemoryUpdateType) -> UpdateInterval {
        let (start, stop) = Utility::get_start_and_stop(memory_update);
        UpdateInterval{ start, stop, val: memory_update.clone() }
    }
}