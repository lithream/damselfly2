//! Represents the status of a block of memory.
use serde::{Serialize, Serializer};
use std::fmt::{Display, Formatter};
use std::mem;
use std::sync::Arc;

/// State of a block of memory.
/// Parent address is the address of the memory update responsible for giving this block its 
/// current state.
/// Address is the address of the block itself.
#[derive(Debug, Clone)]
pub enum MemoryStatus {
    /// parent address, total size, address, callstack
    Allocated(usize, usize, usize, Arc<String>),
    /// parent address, total size, address, callstack
    PartiallyAllocated(usize, usize, usize, Arc<String>),
    /// parent address, total size, address, callstack
    Free(usize, usize, usize, Arc<String>),
    /// address
    Unused(usize),
}

impl PartialEq for MemoryStatus {
    fn eq(&self, other: &Self) -> bool {
        mem::discriminant(self) == mem::discriminant(other)
            && self.get_parent_address() == other.get_parent_address()
    }
}

impl Serialize for MemoryStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}

impl MemoryStatus {
    pub fn get_parent_address(&self) -> Option<usize> {
        match self {
            MemoryStatus::Allocated(parent_address, _, _, _) => Some(*parent_address),
            MemoryStatus::PartiallyAllocated(parent_address, _, _, _) => Some(*parent_address),
            MemoryStatus::Free(parent_address, _, _, _) => Some(*parent_address),
            MemoryStatus::Unused(_) => None,
        }
    }
    
    pub fn get_address(&self) -> usize {
        match self {
            MemoryStatus::Allocated(_, _, address, _) => *address,
            MemoryStatus::PartiallyAllocated(_, _, address, _) => *address,
            MemoryStatus::Free(_, _, address, _) => *address,
            MemoryStatus::Unused(address) => *address
        }
    }
}

impl Display for MemoryStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            MemoryStatus::Allocated(parent_address, size, _address, callstack) => {
                format!("A {} {} {}", parent_address, size, callstack)
            }
            MemoryStatus::PartiallyAllocated(parent_address, size, _address, callstack) => {
                format!("P {} {} {}", parent_address, size, callstack)
            }
            MemoryStatus::Free(parent_address, size, _address, callstack) => {
                format!("F {} {} {}", parent_address, size, callstack)
            }
            MemoryStatus::Unused(_address) => "U".to_string(),
        };
        write!(f, "{}", str)
    }
}
