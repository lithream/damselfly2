use std::sync::Arc;

#[derive(PartialEq, Debug, Clone)]
pub enum MemoryStatus {
    // parent block, allocation size from parent, callstack
    Allocated(usize, usize, Arc<String>),
    // parent block, callstack
    PartiallyAllocated(usize, Arc<String>),
    Free(Arc<String>),
    Unused,
}