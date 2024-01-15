use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use crate::memory::{MemoryStatus, MemoryUpdate};
use crate::damselfly::instruction::Instruction;

pub mod instruction;
mod damselfly_viewer;

struct Damselfly {
    rx: Receiver<Instruction>,
    memory_map: HashMap<i64, MemoryStatus>,
    instruction_history: Vec<Instruction>,
}

impl Damselfly {
    pub fn new(rx: Receiver<Instruction>) -> Damselfly {
        Damselfly {
            rx,
            memory_map: HashMap::new(),
            instruction_history: Vec::new(),
        }
    }

    pub fn execute_instruction(&mut self) {
        let instruction = self.rx.recv().expect("[Damselfly::execute_instruction]: Error receiving from channel");
        match instruction.get_operation() {
            MemoryUpdate::Allocation(address, callstack) => {
                self.memory_map.entry(address)
                    .and_modify(|memory_state| *memory_state = MemoryStatus::Allocated(callstack.clone()))
                    .or_insert(MemoryStatus::Allocated(callstack));
            }
            MemoryUpdate::PartialAllocation(address, callstack) => {
                self.memory_map.entry(address)
                    .and_modify(|memory_state| *memory_state = MemoryStatus::PartiallyAllocated(callstack.clone()))
                    .or_insert(MemoryStatus::PartiallyAllocated(callstack));
            }
            MemoryUpdate::Free(address, callstack) => {
                self.memory_map.entry(address)
                    .and_modify(|memory_state| *memory_state = MemoryStatus::Free(callstack.clone()))
                    .or_insert(MemoryStatus::Free(callstack));
            }
        }
        self.instruction_history.push(instruction);
    }

    pub fn query_block(&self, address: i64) -> Option<&MemoryStatus> {
        self.memory_map.get(&address)
    }

    pub fn get_latest_instruction(&self) -> Option<&Instruction> {
        self.instruction_history.last()
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