use std::collections::HashMap;
use std::fmt::Display;
use std::rc::Rc;
use nohash_hasher::BuildNoHashHasher;
use rust_lapper::{Interval, Lapper};
use crate::damselfly::instruction::Instruction;

pub type NoHashMap<K, V> = HashMap<K, V, BuildNoHashHasher<K>>;

#[derive(Debug, Default, Clone)]
pub struct MemoryUsage {
    pub memory_used_absolute: usize,
    pub total_memory: usize,
    pub blocks: usize,
    pub latest_operation: usize,
}

#[derive(PartialEq, Debug, Clone)]
pub enum MemoryUpdate {
    // (address, size, callstack)
    Allocation(usize, usize, Rc<String>),
    // (address, size, callstack)
    Free(usize, usize, Rc<String>),
}

#[derive(Clone)]
pub enum RecordType {
    // (address, size, callstack)
    Allocation(usize, usize, String),
    // (address, callstack)
    Free(usize, String),
    // (address, callstack)
    StackTrace(usize, String),
}

#[derive(PartialEq, Debug, Clone)]
pub enum MemoryStatus {
    // parent block, allocation size from parent, callstack
    Allocated(usize, usize, Rc<String>),
    // parent block, callstack
    PartiallyAllocated(usize, Rc<String>),
    Free(Rc<String>),
}

impl Display for MemoryUpdate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            MemoryUpdate::Allocation(address, size, _) => format!("ALLOC: 0x{:x} {}B", address, size),
            MemoryUpdate::Free(address, size, _) => format!("FREE: 0x{:x} {}", address, size),
        };
        write!(f, "{}", str)
    }
}

#[derive(Debug)]
pub struct MemorySnapshot {
    pub memory_usage: (f64, usize),
    pub operation: MemoryUpdate,
}

