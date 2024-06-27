use crate::damselfly::memory::memory_update::{MemoryUpdate, MemoryUpdateType};
use crate::damselfly::update_interval::UpdateInterval;

pub struct Utility {}

impl Utility {
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
    
    pub fn get_canvas_span(update_intervals: &Vec<UpdateInterval>) -> (usize, usize) {
        let mut min = usize::MAX;
        let mut max = usize::MIN;
        for update in update_intervals {
            min = std::cmp::min(min, update.val.get_start());
            max = std::cmp::max(max, update.val.get_end());
        }
        (min, max)
    }
    
    pub fn convert_intervals_to_updates<'a>(intervals: &'a Vec<&UpdateInterval>) -> Vec<&'a MemoryUpdateType> {
        let mut update_vec = Vec::new();
        for interval in intervals {
            update_vec.push(&interval.val);
        }
        update_vec
    }

    pub fn clone_intervals_to_update(intervals: &Vec<&UpdateInterval>) -> Vec<MemoryUpdateType> {
        let mut update_vec = Vec::new();
        for interval in intervals {
            update_vec.push(interval.val.clone())
        }
        update_vec
    }
}