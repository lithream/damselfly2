use std::fmt::{Display, Formatter};
use std::sync::Arc;

#[derive(PartialEq, Eq, Clone)]
pub enum MemoryUpdateType {
    Allocation(Allocation),
    Free(Free)
}

impl Display for MemoryUpdateType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str;
        match self {
            MemoryUpdateType::Allocation(allocation) =>
                str = allocation.to_string(),
            MemoryUpdateType::Free(free) =>
                str = free.to_string(),
        };
        write!(f, "{}", str)
    }
}

pub trait MemoryUpdate {
    fn get_absolute_address(&self) -> usize;
    fn get_absolute_size(&self) -> usize;
    fn get_callstack(&self) -> Arc<String>;
    fn get_timestamp(&self) -> usize;
    fn wrap_in_enum(self) -> MemoryUpdateType;
}

#[derive(PartialEq, Eq, Clone)]
pub struct Allocation {
    address: usize,
    size: usize,
    callstack: Arc<String>,
    timestamp: usize,
}

impl Allocation {
    pub fn new(address: usize, size: usize, callstack: Arc<String>, timestamp: usize) -> Allocation {
        Allocation {
            address,
            size,
            callstack,
            timestamp
        }
    }
}

#[derive(PartialEq, Eq, Clone)]
pub struct Free {
    address: usize,
    size: usize,
    callstack: Arc<String>,
    timestamp: usize,
}

impl Free {
    pub fn new(address: usize, size: usize, callstack: Arc<String>, timestamp: usize) -> Free {
        Free {
            address,
            size,
            callstack,
            timestamp
        }
    }
}
impl MemoryUpdate for Allocation {
    fn get_absolute_address(&self) -> usize {
        self.address
    }

    fn get_absolute_size(&self) -> usize {
        self.size
    }

    fn get_callstack(&self) -> Arc<String> {
        Arc::clone(&(self.callstack))
    }

    fn get_timestamp(&self) -> usize {
        self.timestamp
    }

    fn wrap_in_enum(self) -> MemoryUpdateType {
        MemoryUpdateType::Allocation(self)
    }
}

impl MemoryUpdate for Free {
    fn get_absolute_address(&self) -> usize {
        self.address
    }

    fn get_absolute_size(&self) -> usize {
        self.size
    }

    fn get_callstack(&self) -> Arc<String> {
        Arc::clone(&(self.callstack))
    }

    fn get_timestamp(&self) -> usize {
        self.timestamp
    }

    fn wrap_in_enum(self) -> MemoryUpdateType {
        MemoryUpdateType::Free(self)
    }
}

impl Display for Allocation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = format!("[{}] ALLOC: 0x{:x} {}B",
                          self.get_timestamp(),
                          self.get_absolute_address(),
                          self.get_absolute_size());
        write!(f, "{}", str)
    }
}

impl Display for Free {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = format!("[{}] FREE: 0x{:x} {}",
                          self.get_timestamp(),
                          self.get_absolute_address(),
                          self.get_absolute_size());
        write!(f, "{}", str)
    }
}
