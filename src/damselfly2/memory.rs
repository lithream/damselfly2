use std::collections::HashMap;
use nohash_hasher::BuildNoHashHasher;

pub type NoHashMap<K, V> = HashMap<K, V, BuildNoHashHasher<K>>;
pub mod memory_update;
pub mod memory_usage;
mod memory_parsers;
mod memory_usage_factory;

