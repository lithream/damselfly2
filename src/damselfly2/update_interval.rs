use rust_lapper::Interval;
use crate::damselfly2::memory::memory_update::{MemoryUpdate, MemoryUpdateType};

pub type UpdateInterval = Interval<usize, MemoryUpdateType>;
pub mod update_interval_factory;
mod update_interval_sorter;
mod overlap_finder;
mod distinct_block_counter;
mod utility;
mod update_queue_compressor;

