use rust_lapper::Interval;
use crate::damselfly2::memory::memory_update::MemoryUpdateType;

pub type UpdateInterval = Interval<usize, MemoryUpdateType>;
pub mod update_interval_factory;
mod update_interval_sorter;
