use std::collections::HashMap;
use std::fmt::Display;
use std::num::ParseFloatError;
use std::sync::{mpsc};
use std::sync::mpsc::{Receiver, Sender};
use rand::{Rng};
use crate::damselfly_viewer::consts::DEFAULT_MEMORY_SIZE;
use crate::damselfly_viewer::instruction::Instruction;

#[derive(PartialEq, Debug, Clone)]
pub enum MemoryUpdate {
    Allocation(usize, String),
    PartialAllocation(usize, String),
    Free(usize, String),
    Disconnect(String)
}

#[derive(PartialEq, Debug, Clone)]
pub enum MemoryStatus {
    Allocated(String),
    PartiallyAllocated(String),
    Free(String),
}

impl Display for MemoryUpdate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            MemoryUpdate::Allocation(address, callstack) => String::from(format!("ALLOC: {}", address)),
            MemoryUpdate::PartialAllocation(address, callstack) => String::from(format!("P-ALLOC: {}", address)),
            MemoryUpdate::Free(address, callstack) => String::from(format!("FREE: {}", address)),
            MemoryUpdate::Disconnect(_) => String::from("DISCONNECT")
        };
        write!(f, "{}", str)
    }
}

pub trait MemoryTracker {
    fn get_recv(&self) -> Receiver<Instruction>;
}

#[derive(Debug)]
pub struct MemorySnapshot {
    pub memory_usage: (f64, usize),
    pub operation: MemoryUpdate,
}

pub struct MemorySysTraceParser {
    instruction_tx: Sender<Instruction>,
    map: HashMap<usize, MemoryStatus>,
    time: usize
}

impl MemorySysTraceParser {
    pub fn new() -> (MemorySysTraceParser, Receiver<Instruction>) {
        let (tx, rx) = mpsc::channel();
        (MemorySysTraceParser { instruction_tx: tx, map: HashMap::new(), time: 0 }, rx)
    }

    pub fn parse_log(&mut self, log: &str) -> Vec<Instruction> {
        Vec::new()
    }
    fn parse_line(line: &str) -> Result<Instruction, &'static str> {
        let split_line: Vec<&str> = line.trim().split(' ').collect();
        let timestamp = split_line.first();
        if timestamp.is_none() {
            return Err("[MemorySysTraceParser::parse_line]: No timestamp found");
        }

        let binding = line.split('>').collect::<Vec<_>>();
        let dataline = binding.get(1);
        if dataline.is_none() {
            return Err("[MemorySysTraceParser::parse_line]: Failed to split by > char");
        }
        let dataline = dataline.unwrap();

        let split_dataline = dataline.split(' ').collect::<Vec<_>>();
        if split_dataline.len() < 2 || split_dataline.len() > 3 {
            return Err ("[MemorySysTraceParser::parse_line]: Line length mismatch");
        }

        let mut operation;
        match split_dataline[0] {
            "+" => operation = MemoryUpdate::Allocation(0, String::from("default")),
            "-" => operation = MemoryUpdate::Free(0, String::from("default")),
             _  => return Err("[MemorySysTraceParser::parse_line]: Invalid operation type"),
        }

        let address = usize::from_str_radix(split_dataline[1], 16);
        if let Err(error) = address {
            return Err("[MemorySysTraceParser::parse_line]: Failed to convert address to decimal");
        }

        let address = address.unwrap();
        match operation {
            MemoryUpdate::Allocation(ref mut stored_address, _) => *stored_address = address,
            MemoryUpdate::Free(ref mut stored_address , _) => *stored_address = address,
            _ => return Err("[MemorySysTraceParser::parse_line]: Invalid operation type"),
        }

        if let MemoryUpdate::Allocation(_, ) = operation {
            let allocated_bytes = split_dataline[2];
            operation.
        }
        Err("defaul")
    }
}

pub struct MemoryStub {
    instruction_tx: Sender<Instruction>,
    map: HashMap<usize, MemoryStatus>,
    time: usize
}

impl MemoryStub {
    pub fn new() -> (MemoryStub, Receiver<Instruction>) {
        let (tx, rx) = mpsc::channel();
        (MemoryStub { instruction_tx: tx, map: HashMap::new(), time: 0 }, rx)
    }

    pub fn generate_event_sequential(&mut self) {
        self.map.insert(self.time, MemoryStatus::Allocated(String::from("generate_event_sequential_Allocation")));
        let instruction = Instruction::new(self.time, MemoryUpdate::Allocation(self.time, String::from("generate_event_sequential_Allocation")));
        self.instruction_tx.send(instruction).unwrap();
        self.time += 1;
    }

    pub fn generate_event(&mut self) {
        let address: usize = rand::thread_rng().gen_range(0..DEFAULT_MEMORY_SIZE);
        let block_size = rand::thread_rng().gen_range(0..64);
            match rand::thread_rng().gen_range(0..3) {
                0 => {
                    for cur_address in address..address + block_size {
                        self.map.insert(cur_address, MemoryStatus::Allocated(String::from("generate_event_Allocation")));
                        let instruction = Instruction::new(cur_address, MemoryUpdate::Allocation(cur_address, String::from("generate_event_Allocation")));
                        self.instruction_tx.send(instruction).unwrap();
                    }
                },
                1 => {
                    for cur_address in address..address + block_size {
                        self.map.insert(cur_address, MemoryStatus::PartiallyAllocated(String::from("generate_event_PartialAllocation")));
                        let instruction = Instruction::new(cur_address, MemoryUpdate::PartialAllocation(cur_address, String::from("generate_event_PartialAllocation")));
                        self.instruction_tx.send(instruction).unwrap();
                    }
                },
                2 => {
                    for cur_address in address..address + block_size {
                        self.map.insert(cur_address, MemoryStatus::Free(String::from("generate_event_Free")));
                        let instruction = Instruction::new(cur_address, MemoryUpdate::Free(cur_address, String::from("generate_event_Free")));
                        self.instruction_tx.send(instruction).unwrap();
                    }
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
    use log::debug;
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
                    debug!("[EVENT #{i}: SUCCESS]: Allocation({address} {callstack})");
                }
                MemoryUpdate::PartialAllocation(address, callstack) => {
                    debug!("[EVENT #{i}: SUCCESS]: PartialAllocation({address} {callstack})");
                }
                MemoryUpdate::Free(address, callstack) => {
                    debug!("[EVENT #{i}: SUCCESS]: Free({address} {callstack})");
                }
                MemoryUpdate::Disconnect(reason) => {
                    debug!("[EVENT #{i}: SUCCESS]: Disconnect({reason})");
                }
            }
        }
    }
}