use std::collections::HashMap;
use std::sync::mpsc;
use crate::memory::{MemorySnapshot, MemoryStatus, MemoryUpdate};
use crate::damselfly::instruction::Instruction;

pub mod instruction;
mod damselfly_viewer;

const MAX_MEMORY: u64 = 65535;
struct Damselfly {
    instruction_rx: mpsc::Receiver<Instruction>,
    snapshot_tx: mpsc::Sender<MemorySnapshot>,
    memory_map: HashMap<u64, MemoryStatus>,
}

impl Damselfly {
    pub fn new(instruction_rx: mpsc::Receiver<Instruction>) -> (Damselfly, mpsc::Receiver<MemorySnapshot>) {
        let (snapshot_tx, snapshot_rx) = mpsc::channel::<MemorySnapshot>();
        (
            Damselfly {
            instruction_rx,
            snapshot_tx,
            memory_map: HashMap::new(),
        },
            snapshot_rx
        )
    }

    pub fn execute_instruction(&mut self) {
        let instruction = self.instruction_rx.recv().expect("[Damselfly::execute_instruction]: Error receiving from channel");
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
            MemoryUpdate::Disconnect(reason) => {
                println!("[Damselfly::execute_instruction]: Memory disconnected ({reason})");
            }
        }
        self.send_snapshot();
    }

    pub fn get_memory_usage(&self) -> (f64, u64) {
        let mut memory_usage: f64 = 0.0;
        for address in self.memory_map.keys() {
            match self.memory_map.get(address).unwrap() {
                MemoryStatus::Allocated(_) => memory_usage += 1.0,
                MemoryStatus::PartiallyAllocated(_) => memory_usage += 0.5,
                MemoryStatus::Free(_) => {}
            }
        }
        (memory_usage, MAX_MEMORY)
    }

    pub fn send_snapshot(&self) {
        let snapshot = MemorySnapshot{
            memory_usage: self.get_memory_usage(),
            memory_map: self.memory_map.clone()
        };
        self.snapshot_tx.send(snapshot).expect("[Damselfly::send_snapshot]: Error sending into snapshot channel");
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
        let (mut damselfly, _snapshot_rx) = Damselfly::new(rx);
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