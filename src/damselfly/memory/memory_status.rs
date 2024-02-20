use std::mem;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum MemoryStatus {
    // parent address, total size, callstack
    Allocated(usize, usize, Arc<String>),
    PartiallyAllocated(usize, usize, Arc<String>),
    Free(usize, usize, Arc<String>),
    Unused,
}

impl PartialEq for MemoryStatus {
    fn eq(&self, other: &Self) -> bool {
        mem::discriminant(self) == mem::discriminant(other) &&
            self.get_parent_address() == other.get_parent_address()
    }
}

impl MemoryStatus {
    pub fn get_parent_address(&self) -> Option<usize> {
        match self {
            MemoryStatus::Allocated(parent_address, _, _) => Some(*parent_address),
            MemoryStatus::PartiallyAllocated(parent_address, _, _) => Some(*parent_address),
            MemoryStatus::Free(parent_address, _, _) => Some(*parent_address),
            MemoryStatus::Unused => None,
        }
    }
}