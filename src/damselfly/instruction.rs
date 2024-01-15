use crate::memory::MemoryUpdate;

#[derive(Clone, Debug)]
pub struct Instruction {
    timestamp: u64,
    operation: MemoryUpdate
}

impl Instruction {
    pub fn new(timestamp: u64, operation: MemoryUpdate) -> Instruction {
        Instruction { timestamp, operation }
    }

    pub fn get_operation(&self) -> MemoryUpdate {
        self.operation.clone()
    }

    pub fn get_timestamp(&self) -> u64 {
        self.timestamp
    }
}