use std::collections::HashMap;
use nohash_hasher::BuildNoHashHasher;

pub type NoHashMap<K, V> = HashMap<K, V, BuildNoHashHasher<K>>;
pub mod memory_update;
pub mod memory_usage;
pub mod memory_parsers;
pub mod memory_usage_factory;
pub mod memory_status;
mod update_sampler;
mod memory_cache;

