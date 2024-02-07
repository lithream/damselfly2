use crate::damselfly::memory_structs::MemoryUpdate;

#[derive(Clone, Debug)]
pub struct Instruction {
    timestamp: usize,
    operation: MemoryUpdate
}

impl Instruction {
    pub fn new(timestamp: usize, operation: MemoryUpdate) -> Instruction {
        Instruction { timestamp, operation }
    }

    pub fn get_operation(&self) -> MemoryUpdate {
        self.operation.clone()
    }

    pub fn get_timestamp(&self) -> usize {
        self.timestamp
    }
}