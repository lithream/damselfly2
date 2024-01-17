use std::collections::HashMap;
use std::sync::mpsc;
use log::debug;
use crate::memory::{MemoryStatus, MemoryUpdate};
use crate::damselfly::instruction::Instruction;

pub mod instruction;
pub mod damselfly_viewer;

const DEFAULT_MEMORY_SIZE: usize = 4096;
pub struct Damselfly {
    instruction_rx: mpsc::Receiver<Instruction>,
    memory_map: HashMap<usize, MemoryStatus>,
    operation_history: Vec<MemoryUpdate>,
}

impl Damselfly {
    pub fn new(instruction_rx: mpsc::Receiver<Instruction>) -> Damselfly {
        Damselfly {
            instruction_rx,
            memory_map: HashMap::new(),
            operation_history: Vec::new()
        }
    }

    pub fn execute_instruction(&mut self) {
        let operation = self.instruction_rx.recv().expect("[Damselfly::execute_instruction]: Error receiving from channel").get_operation();
        match operation {
            MemoryUpdate::Allocation(address, ref callstack) => {
                self.memory_map.entry(address)
                    .and_modify(|memory_state| *memory_state = MemoryStatus::Allocated(callstack.clone()))
                    .or_insert(MemoryStatus::Allocated(callstack.clone()));
                self.operation_history.push(operation);
            }
            MemoryUpdate::PartialAllocation(address, ref callstack) => {
                self.memory_map.entry(address)
                    .and_modify(|memory_state| *memory_state = MemoryStatus::PartiallyAllocated(callstack.clone()))
                    .or_insert(MemoryStatus::PartiallyAllocated(callstack.clone()));
                self.operation_history.push(operation);
            }
            MemoryUpdate::Free(address, ref callstack) => {
                self.memory_map.entry(address)
                    .and_modify(|memory_state| *memory_state = MemoryStatus::Free(callstack.clone()))
                    .or_insert(MemoryStatus::Free(callstack.clone()));
                self.operation_history.push(operation);
            }
            MemoryUpdate::Disconnect(reason) => {
                debug!("[Damselfly::execute_instruction]: Memory disconnected ({reason})");
            }
        }
    }

    pub fn get_memory_usage(&self) -> (f64, usize) {
        let mut memory_usage: f64 = 0.0;
        for address in self.memory_map.keys() {
            if let Some(memory_status) = self.memory_map.get(address) {
                match memory_status {
                    MemoryStatus::Allocated(_) => memory_usage += 1.0,
                    MemoryStatus::PartiallyAllocated(_) => memory_usage += 0.5,
                    MemoryStatus::Free(_) => {}
                }
            } else {
                return (0.0, DEFAULT_MEMORY_SIZE);
            }
        }
        (memory_usage, DEFAULT_MEMORY_SIZE)
    }

    pub fn get_latest_map_state(&self) -> &HashMap<usize, MemoryStatus>{
        &self.memory_map
    }

    pub fn get_map_state(&self, time: usize) -> HashMap<usize, MemoryStatus> {
        let mut map: HashMap<usize, MemoryStatus> = HashMap::new();
        let mut iter = self.operation_history.iter();
        for _ in 0..=time {
            if let Some(operation) = iter.next() {
                match operation {
                    MemoryUpdate::Allocation(address, callstack) => {
                        map.insert(*address, MemoryStatus::Allocated(String::from(callstack)));
                    }
                    MemoryUpdate::PartialAllocation(address, callstack) => {
                        map.insert(*address, MemoryStatus::PartiallyAllocated(String::from(callstack)));
                    }
                    MemoryUpdate::Free(address, callstack) => {
                        map.insert(*address, MemoryStatus::Free(String::from(callstack)));
                    }
                    MemoryUpdate::Disconnect(_) => {
                        // nothing
                    }
                }
            }
        }
        map
    }
}

#[cfg(test)]
mod tests {
    use std::thread;
    use crate::damselfly::Damselfly;
    use crate::memory::{MemoryStatus, MemoryStub, MemoryUpdate};

    #[test]
    fn link_memory_stub_to_damselfly() {
        let (mut memory_stub, rx) = MemoryStub::new();
        thread::spawn(move || {
            for i in 0..3 {
                memory_stub.force_generate_event(MemoryUpdate::Allocation(i, String::from("force_generate_event_Allocation")))
            }
            for i in 3..6 {
                memory_stub.force_generate_event(MemoryUpdate::PartialAllocation(i, String::from("force_generate_event_PartialAllocation")));
            }
            for i in 1..4 {
                memory_stub.force_generate_event(MemoryUpdate::Free(i, String::from("force_generate_event_Free")));
            }
        });
        let mut damselfly = Damselfly::new(rx);
        for _ in 0..9 {
            damselfly.execute_instruction()
        }
        assert_eq!(*damselfly.memory_map.get(&0).unwrap(), MemoryStatus::Allocated(String::from("force_generate_event_Allocation")));
        assert_eq!(*damselfly.memory_map.get(&1).unwrap(), MemoryStatus::Free(String::from("force_generate_event_Free")));
        assert_eq!(*damselfly.memory_map.get(&2).unwrap(), MemoryStatus::Free(String::from("force_generate_event_Free")));
        assert_eq!(*damselfly.memory_map.get(&3).unwrap(), MemoryStatus::Free(String::from("force_generate_event_Free")));
        assert_eq!(*damselfly.memory_map.get(&4).unwrap(), MemoryStatus::PartiallyAllocated(String::from("force_generate_event_PartialAllocation")));
        assert_eq!(*damselfly.memory_map.get(&5).unwrap(), MemoryStatus::PartiallyAllocated(String::from("force_generate_event_PartialAllocation")));
    }
}