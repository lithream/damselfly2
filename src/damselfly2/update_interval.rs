use rust_lapper::Interval;
use crate::damselfly2::memory::memory_update::{MemoryUpdate, MemoryUpdateType};

pub type UpdateInterval = Interval<usize, MemoryUpdateType>;
pub mod update_interval_factory;
pub mod update_interval_sorter;
pub mod overlap_finder;
pub mod distinct_block_counter;
pub mod utility;
pub mod update_queue_compressor;

