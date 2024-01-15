use std::collections::HashMap;
use std::sync::{mpsc};
use std::sync::mpsc::{Receiver, Sender};
use log::debug;
use rand::{Rng};
use crate::damselfly::instruction::Instruction;

#[derive(PartialEq, Debug, Clone)]
pub enum MemoryUpdate {
    Allocation(u64, String),
    PartialAllocation(u64, String),
    Free(u64, String),
    Disconnect(String)
}

#[derive(PartialEq, Debug, Clone)]
pub enum MemoryStatus {
    Allocated(String),
    PartiallyAllocated(String),
    Free(String),
}

pub trait MemoryTracker {
    fn get_recv(&self) -> Receiver<Instruction>;
}

#[derive(Debug)]
pub struct MemorySnapshot {
    pub memory_usage: (f64, u64),
    pub memory_map: HashMap<u64, MemoryStatus>
}

pub struct MemoryStub {
    instruction_tx: Sender<Instruction>,
    map: HashMap<u64, MemoryStatus>,
    time: u64
}

impl MemoryStub {
    pub fn new() -> (MemoryStub, Receiver<Instruction>) {
        let (tx, rx) = mpsc::channel();
        (MemoryStub { instruction_tx: tx, map: HashMap::new(), time: 0 }, rx)
    }

    pub fn generate_event(&mut self) {
        let address: u64 = rand::thread_rng().gen_range(0..65536);
            match rand::thread_rng().gen_range(0..3) {
                0 => {
                    self.map.insert(address, MemoryStatus::Allocated(String::from("generate_event_Allocation")));
                    let instruction = Instruction::new(self.time, MemoryUpdate::Allocation(address, String::from("generate_event_Allocation")));
                    dbg!(&instruction);
                    self.instruction_tx.send(instruction).unwrap();
                },
                1 => {
                    self.map.insert(address, MemoryStatus::PartiallyAllocated(String::from("generate_event_PartialAllocation")));
                    let instruction = Instruction::new(self.time, MemoryUpdate::PartialAllocation(address, String::from("generate_event_PartialAllocation")));
                    dbg!(&instruction);
                    self.instruction_tx.send(instruction).unwrap();
                },
                2 => {
                    self.map.insert(address, MemoryStatus::Free(String::from("generate_event_Free")));
                    let instruction = Instruction::new(self.time, MemoryUpdate::Free(address, String::from("generate_event_Free")));
                    dbg!(&instruction);
                    self.instruction_tx.send(instruction).unwrap();
                },
                _ => { panic!("[MemoryStub::generate_event]: Thread RNG out of scope") }
            };
        self.time += 1;
    }

    pub fn force_generate_event(&mut self, event: MemoryUpdate) {
        match event {
            MemoryUpdate::Allocation(address, ref callstack) => {
                self.map.insert(address, MemoryStatus::Allocated(callstack.clone()));
                let instruction = Instruction::new(self.time, event);
                self.instruction_tx.send(instruction).unwrap();
            }
            MemoryUpdate::PartialAllocation(address, ref callstack) => {
                self.map.insert(address, MemoryStatus::PartiallyAllocated(callstack.clone()));
                let instruction = Instruction::new(self.time, event);
                self.instruction_tx.send(instruction).unwrap();

            }
            MemoryUpdate::Free(address, ref callstack) => {
                self.map.insert(address, MemoryStatus::Free(callstack.clone()));
                let instruction = Instruction::new(self.time, event);
                self.instruction_tx.send(instruction).unwrap();

            }
            MemoryUpdate::Disconnect(reason) => {
                let instruction = Instruction::new(self.time, MemoryUpdate::Disconnect(reason));
                self.instruction_tx.send(instruction).unwrap();
            }
        }
        self.time += 1;
    }
}

#[cfg(test)]
mod tests {
    use crate::memory::{MemoryStatus, MemoryStub, MemoryUpdate};

    #[test]
    fn allocate() {
        let (mut memory_stub, rx) = MemoryStub::new();
        memory_stub.force_generate_event(MemoryUpdate::Allocation(0, String::from("force_generate_event_Allocation")));
        assert_eq!(*memory_stub.map.get(&0).unwrap(), MemoryStatus::Allocated(String::from("force_generate_event_Allocation")));
        assert_eq!(rx.recv().unwrap().get_operation(), MemoryUpdate::Allocation(0, String::from("force_generate_event_Allocation")));
    }

    #[test]
    fn partially_allocate() {
        let (mut memory_stub, rx) = MemoryStub::new();
        memory_stub.force_generate_event(MemoryUpdate::PartialAllocation(0, String::from("force_generate_event_PartialAllocation")));
        assert_eq!(*memory_stub.map.get(&0).unwrap(), MemoryStatus::PartiallyAllocated(String::from("force_generate_event_PartialAllocation")));
        assert_eq!(rx.recv().unwrap().get_operation(), MemoryUpdate::PartialAllocation(0, String::from("force_generate_event_PartialAllocation")));
    }

    #[test]
    fn free() {
        let (mut memory_stub, rx) = MemoryStub::new();
        memory_stub.force_generate_event(MemoryUpdate::Allocation(0, String::from("force_generate_event_Free")));
        assert_eq!(*memory_stub.map.get(&0).unwrap(), MemoryStatus::Allocated(String::from("force_generate_event_Free")));
        memory_stub.force_generate_event(MemoryUpdate::Free(0, String::from("force_generate_event_Free")));
        assert_eq!(*memory_stub.map.get(&0).unwrap(), MemoryStatus::Free(String::from("force_generate_event_Free")));
        assert_eq!(rx.recv().unwrap().get_operation(), MemoryUpdate::Allocation(0, String::from("force_generate_event_Free")));
        assert_eq!(rx.recv().unwrap().get_operation(), MemoryUpdate::Free(0, String::from("force_generate_event_Free")));
    }

    #[test]
    fn generate_random_events() {
        let (mut memory_stub, rx) = MemoryStub::new();
        for i in 0..10 {
            memory_stub.generate_event();
            let event = rx.recv().unwrap().get_operation();
            match event {
                MemoryUpdate::Allocation(address, callstack) => {
                    eprintln!("[EVENT #{i}: SUCCESS]: Allocation({address} {callstack})");
                }
                MemoryUpdate::PartialAllocation(address, callstack) => {
                    eprintln!("[EVENT #{i}: SUCCESS]: PartialAllocation({address} {callstack})");
                }
                MemoryUpdate::Free(address, callstack) => {
                    eprintln!("[EVENT #{i}: SUCCESS]: Free({address} {callstack})");
                }
                MemoryUpdate::Disconnect(reason) => {
                    eprintln!("[EVENT #{i}: SUCCESS]: Disconnect({reason})");
                }
            }
        }
    }
}