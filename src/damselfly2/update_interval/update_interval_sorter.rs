use crate::damselfly2::memory::memory_update::{MemoryUpdate, MemoryUpdateType};
use super::UpdateInterval;

pub struct UpdateIntervalSorter;

impl UpdateIntervalSorter {
    pub fn sort_by_timestamp(intervals: &mut Vec<&UpdateInterval>) {
        intervals.sort_unstable_by(|prev, next| {
            let get_timestamp = |update_interval: &&UpdateInterval| -> usize {
                match &update_interval.val {
                    MemoryUpdateType::Allocation(allocation) => allocation.get_timestamp(),
                    MemoryUpdateType::Free(free) => free.get_timestamp(),
                }
            };

            get_timestamp(prev).cmp(&get_timestamp(next))
        })
    }
}
