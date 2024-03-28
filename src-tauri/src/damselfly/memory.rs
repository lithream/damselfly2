use std::collections::HashMap;
use nohash_hasher::BuildNoHashHasher;

pub type NoHashMap<K, V> = HashMap<K, V, BuildNoHashHasher<K>>;
pub mod memory_update;
pub mod memory_usage;
pub mod memory_parsers;
pub mod memory_usage_factory;
pub mod memory_status;
pub mod update_sampler;
pub mod memory_cache;
pub mod memory_cache_snapshot;
pub(crate) mod utility;

