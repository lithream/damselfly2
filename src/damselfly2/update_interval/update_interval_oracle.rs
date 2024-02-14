use rust_lapper::{Interval, Lapper};
use crate::damselfly2::memory::memory_update::MemoryUpdateType;
use crate::damselfly2::update_interval::UpdateInterval;

pub struct UpdateIntervalOracle {
    lapper: Lapper<usize, MemoryUpdateType>
}

impl UpdateIntervalOracle {
    pub fn new(intervals: Vec<Interval<usize, MemoryUpdateType>>) -> UpdateIntervalOracle {
        UpdateIntervalOracle {
            lapper: Lapper::new(intervals)
        }
    }

    // start..end, not start..=end
    pub fn find_overlaps(&self, start: usize, end: usize) -> Vec<&UpdateInterval> {
        self.lapper.find(start, end).collect::<Vec<&UpdateInterval>>()
    }
}