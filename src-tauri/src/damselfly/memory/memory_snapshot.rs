use rust_lapper::Lapper;
use crate::damselfly::memory::memory_update::MemoryUpdateType;
use crate::damselfly::update_interval::UpdateInterval;

pub struct MemorySnapshot {
    lapper: Lapper<usize, MemoryUpdateType>
}

impl MemorySnapshot {
    pub fn new(update_intervals: Vec<UpdateInterval>) -> Self {
        Self {
            lapper: Lapper::new(update_intervals)
        }
    }
}