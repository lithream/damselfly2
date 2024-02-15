use std::sync::Arc;

#[derive(PartialEq, Debug, Clone)]
pub enum MemoryStatus {
    // parent address, total size, callstack
    Allocated(usize, usize, Arc<String>),
    PartiallyAllocated(usize, usize, Arc<String>),
    Free(usize, usize, Arc<String>),
    Unused,
}