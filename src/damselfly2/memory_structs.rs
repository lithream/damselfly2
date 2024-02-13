use std::collections::HashMap;
use nohash_hasher::BuildNoHashHasher;

pub type NoHashMap<K, V> = HashMap<K, V, BuildNoHashHasher<K>>;

#[derive(Debug, Default, Clone)]
pub struct MemoryUsage {
    pub memory_used_absolute: usize,
    pub total_memory: usize,
    pub blocks: usize,
    pub latest_operation: usize,
}