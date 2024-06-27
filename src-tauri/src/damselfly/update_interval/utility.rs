//! Utility functions for update_interval.
use crate::damselfly::memory::memory_update::{MemoryUpdate, MemoryUpdateType};
use crate::damselfly::update_interval::UpdateInterval;

pub struct Utility {}

impl Utility {
    /// Gets the start and stop addresses of a MemoryUpdate.
    /// 
    /// # Arguments 
    /// 
    /// * `memory_update`: Memory update.
    /// 
    /// returns: (start, stop) 
    pub fn get_start_and_stop(memory_update: &MemoryUpdateType) -> (usize, usize) {
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
    
    /// Gets the minimum and maximum addresses spanned by a collection of UpdateIntervals.
    pub fn get_canvas_span(update_intervals: &Vec<UpdateInterval>) -> (usize, usize) {
        let mut min = usize::MAX;
        let mut max = usize::MIN;
        for update in update_intervals {
            min = std::cmp::min(min, update.val.get_start());
            max = std::cmp::max(max, update.val.get_end());
        }
        (min, max)
    }
    
    /// Converts UpdateIntervals to MemoryUpdateTypes without cloning - lifetimes are preserved.
    pub fn convert_intervals_to_updates<'a>(intervals: &'a Vec<&UpdateInterval>) -> Vec<&'a MemoryUpdateType> {
        let mut update_vec = Vec::new();
        for interval in intervals {
            update_vec.push(&interval.val);
        }
        update_vec
    }

    /// Converts UpdateIntervals to MemoryUpdateTypes by cloning.
    pub fn clone_intervals_to_update(intervals: &Vec<&UpdateInterval>) -> Vec<MemoryUpdateType> {
        let mut update_vec = Vec::new();
        for interval in intervals {
            update_vec.push(interval.val.clone())
        }
        update_vec
    }
}