use std::sync::Arc;

#[derive(PartialEq, Debug, Clone)]
pub enum MemoryStatus {
    Allocated,
    PartiallyAllocated,
    Free,
    Unused,
}