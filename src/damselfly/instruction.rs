use crate::memory::MemoryUpdate;

#[derive(Copy, Clone)]
pub struct Instruction {
    timestamp: i64,
    operation: MemoryUpdate
}

impl Instruction {
    pub fn new(timestamp: i64, operation: MemoryUpdate) -> Instruction {
        Instruction { timestamp, operation }
    }

    pub fn get_operation(&self) -> MemoryUpdate {
        self.operation
    }

    pub fn get_timestamp(&self) -> i64 {
        self.timestamp
    }
}