use crate::damselfly::memory::memory_update::MemoryUpdateType;
use crate::damselfly::update_interval::UpdateInterval;

pub struct IntervalToUpdateConverter {}

impl IntervalToUpdateConverter {
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