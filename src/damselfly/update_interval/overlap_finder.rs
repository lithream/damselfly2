use rust_lapper::{Interval, Lapper};
use crate::damselfly::memory::memory_update::MemoryUpdateType;
use crate::damselfly::update_interval::UpdateInterval;

pub struct OverlapFinder {
    lapper: Lapper<usize, MemoryUpdateType>
}

impl OverlapFinder {
    pub fn default() -> OverlapFinder {
        OverlapFinder {
            lapper: Lapper::new(vec![])
        }
    }
    
    pub fn new(intervals: Vec<Interval<usize, MemoryUpdateType>>) -> OverlapFinder {
        OverlapFinder {
            lapper: Lapper::new(intervals)
        }
    }

    pub fn push_interval(&mut self, interval: Interval<usize, MemoryUpdateType>) {
        self.lapper.insert(interval);
    }
    
    // start..end, not start..=end
    pub fn find_overlaps(&self, start: usize, end: usize) -> Vec<&UpdateInterval> {
        self.lapper.find(start, end).collect::<Vec<&UpdateInterval>>()
    }
}