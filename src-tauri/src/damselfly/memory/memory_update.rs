use std::fmt::{Display, Formatter};
use std::sync::Arc;

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum MemoryUpdateType {
    Allocation(Allocation),
    Free(Free)
}

impl MemoryUpdateType {
    pub fn get_absolute_address(&self) -> usize {
        match self {
            MemoryUpdateType::Allocation(allocation) => allocation.get_absolute_address(),
            MemoryUpdateType::Free(free) => free.get_absolute_address(),
        }
    }

    pub fn get_absolute_size(&self) -> usize {
        match self {
            MemoryUpdateType::Allocation(allocation) => allocation.get_absolute_size(),
            MemoryUpdateType::Free(free) => free.get_absolute_size(),
        }
    }

    pub fn get_callstack(&self) -> Arc<String> {
        match self {
            MemoryUpdateType::Allocation(allocation) => allocation.get_callstack(),
            MemoryUpdateType::Free(free) => free.get_callstack(),
        }
    }

    pub fn get_start(&self) -> usize {
        match self {
            MemoryUpdateType::Allocation(allocation) => allocation.get_absolute_address(),
            MemoryUpdateType::Free(free) => free.get_absolute_address(),
        }
    }

    pub fn get_end(&self) -> usize {
        match self {
            MemoryUpdateType::Allocation(allocation) => allocation.get_absolute_address() + allocation.get_absolute_size(),
            MemoryUpdateType::Free(free) => free.get_absolute_address() + free.get_absolute_size(),
        }
    }
    
    pub fn get_timestamp(&self) -> usize {
        match self {
            MemoryUpdateType::Allocation(allocation) => allocation.get_timestamp(),
            MemoryUpdateType::Free(free) => free.get_timestamp(),
        }
    }

    pub fn get_real_timestamp(&self) -> &String {
        match self {
            MemoryUpdateType::Allocation(allocation) => allocation.get_real_timestamp(),
            MemoryUpdateType::Free(free) => free.get_real_timestamp(),
        }
    }
}

impl Display for MemoryUpdateType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            MemoryUpdateType::Allocation(allocation) =>
                allocation.to_string(),
            MemoryUpdateType::Free(free) =>
                free.to_string(),
        };
        write!(f, "{}", str)
    }
}

pub trait MemoryUpdate {
    fn get_absolute_address(&self) -> usize;
    fn get_absolute_size(&self) -> usize;
    fn get_callstack(&self) -> Arc<String>;
    fn get_timestamp(&self) -> usize;
    fn get_real_timestamp(&self) -> &String;
    fn wrap_in_enum(self) -> MemoryUpdateType;
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Allocation {
    address: usize,
    size: usize,
    callstack: Arc<String>,
    timestamp: usize,
    real_timestamp: String,
}

impl Allocation {
    pub fn new(address: usize, size: usize, callstack: Arc<String>, timestamp: usize, real_timestamp: String) -> Allocation {
        Allocation {
            address,
            size,
            callstack,
            timestamp,
            real_timestamp,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Free {
    address: usize,
    size: usize,
    callstack: Arc<String>,
    timestamp: usize,
    real_timestamp: String,
}

impl Free {
    pub fn new(address: usize, size: usize, callstack: Arc<String>, timestamp: usize, real_timestamp: String) -> Free {
        Free {
            address,
            size,
            callstack,
            timestamp,
            real_timestamp,
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
    fn get_real_timestamp(&self) -> &String {
        &self.real_timestamp
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
    fn get_real_timestamp(&self) -> &String {
        &self.real_timestamp
    }

    fn wrap_in_enum(self) -> MemoryUpdateType {
        MemoryUpdateType::Free(self)
    }
}

impl Display for Allocation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = format!("[{} {}] ALLOC: 0x{:x} {}B",
                          self.get_timestamp(),
                          self.get_real_timestamp(),
                          self.get_absolute_address(),
                          self.get_absolute_size());
        write!(f, "{}", str)
    }
}

impl Display for Free {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = format!("[{} {}] FREE: 0x{:x} {}B",
                          self.get_timestamp(),
                          self.get_real_timestamp(),
                          self.get_absolute_address(),
                          self.get_absolute_size());
        write!(f, "{}", str)
    }
}
