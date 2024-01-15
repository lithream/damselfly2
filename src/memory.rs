use std::collections::HashMap;
use std::sync::{mpsc};
use std::sync::mpsc::{Receiver, Sender};
use rand::{Rng};
use crate::damselfly::instruction::Instruction;

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum MemoryUpdate {
    Allocation(i64),
    PartialAllocation(i64),
    Free(i64),
}

#[derive(PartialEq, Debug)]
pub enum MemoryStatus {
    Allocated(String),
    PartiallyAllocated(String),
    Free(String),
}
pub trait MemoryTracker {
    fn get_recv(&self) -> Receiver<Instruction>;
}

pub struct MemoryStub {
    tx: Sender<Instruction>,
    map: HashMap<i64, MemoryStatus>,
    time: i64
}

impl MemoryStub {
    pub fn new() -> (MemoryStub, Receiver<Instruction>) {
        let (tx, rx) = mpsc::channel();
        (MemoryStub { tx, map: HashMap::new(), time: -1 }, rx)
    }

    pub fn generate_event(&mut self) {
        self.time += 1;
        let address = rand::thread_rng().gen_range(0..65536);
            match rand::thread_rng().gen_range(0..3) {
                0 => {
                    self.map.insert(address, MemoryStatus::Allocated(String::from("generate_event_0")));
                    let instruction = Instruction::new(self.time, MemoryUpdate::Allocation(address));
                    self.tx.send(instruction).unwrap();
                },
                1 => {
                    self.map.insert(address, MemoryStatus::PartiallyAllocated(String::from("generate_event_1")));
                    let instruction = Instruction::new(self.time, MemoryUpdate::PartialAllocation(address));
                    self.tx.send(instruction).unwrap();
                },
                2 => {
                    self.map.insert(address, MemoryStatus::Free(String::from("generate_event_1")));
                    let instruction = Instruction::new(self.time, MemoryUpdate::Free(address));
                    self.tx.send(instruction).unwrap();
                },
                _ => { panic!("[MemoryStub::generate_event]: Thread RNG out of scope") }
            };
    }

    pub fn force_generate_event(&mut self, event: MemoryUpdate) {
        self.time += 1;
        match event {
            MemoryUpdate::Allocation(address) => {
                self.map.insert(address, MemoryStatus::Allocated);
                let instruction = Instruction::new(self.time, event);
                self.tx.send(instruction).unwrap();
            }
            MemoryUpdate::PartialAllocation(address) => {
                self.map.insert(address, MemoryStatus::PartiallyAllocated);
                let instruction = Instruction::new(self.time, event);
                self.tx.send(instruction).unwrap();

            }
            MemoryUpdate::Free(address) => {
                self.map.insert(address, MemoryStatus::Free);
                let instruction = Instruction::new(self.time, event);
                self.tx.send(instruction).unwrap();

            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::memory::{MemoryStatus, MemoryStub, MemoryUpdate};

    #[test]
    fn allocate() {
        let (mut memory_stub, rx) = MemoryStub::new();
        memory_stub.force_generate_event(MemoryUpdate::Allocation(0));
        assert_eq!(*memory_stub.map.get(&0).unwrap(), MemoryStatus::Allocated);
        assert_eq!(rx.recv().unwrap().get_operation(), MemoryUpdate::Allocation(0));
    }

    #[test]
    fn partially_allocate() {
        let (mut memory_stub, rx) = MemoryStub::new();
        memory_stub.force_generate_event(MemoryUpdate::PartialAllocation(0));
        assert_eq!(*memory_stub.map.get(&0).unwrap(), MemoryStatus::PartiallyAllocated);
        assert_eq!(rx.recv().unwrap().get_operation(), MemoryUpdate::PartialAllocation(0));
    }

    #[test]
    fn free() {
        let (mut memory_stub, rx) = MemoryStub::new();
        memory_stub.force_generate_event(MemoryUpdate::Allocation(0));
        assert_eq!(*memory_stub.map.get(&0).unwrap(), MemoryStatus::Allocated);
        memory_stub.force_generate_event(MemoryUpdate::Free(0));
        assert_eq!(*memory_stub.map.get(&0).unwrap(), MemoryStatus::Free);
        assert_eq!(rx.recv().unwrap().get_operation(), MemoryUpdate::Allocation(0));
        assert_eq!(rx.recv().unwrap().get_operation(), MemoryUpdate::Free(0));
    }

    #[test]
    fn generate_random_events() {
        let (mut memory_stub, mut rx) = MemoryStub::new();
        for i in 0..10 {
            memory_stub.generate_event();
            let event = rx.recv().unwrap().get_operation();
            match event {
                MemoryUpdate::Allocation(addr) => {
                    eprintln!("[EVENT #{i}: SUCCESS]: Allocation({addr})");
                }
                MemoryUpdate::PartialAllocation(addr) => {
                    eprintln!("[EVENT #{i}: SUCCESS]: PartialAllocation({addr})");
                }
                MemoryUpdate::Free(addr) => {
                    eprintln!("[EVENT #{i}: SUCCESS]: Free({addr})");
                }
            }
        }
    }
}