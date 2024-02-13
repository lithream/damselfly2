use rust_lapper::Interval;
use crate::damselfly::instruction::Instruction;
use crate::damselfly::memory_structs::{MemoryUpdate, NoHashMap};

type InstructionInterval = Interval<usize, Instruction>;

pub struct InstructionIntervalFactory {
    instruction_history_map: NoHashMap<usize, Vec<Instruction>>,
}

impl InstructionIntervalFactory {
    pub fn load_instruction(&mut self, instruction: Instruction) {
        match instruction.get_operation() {
            MemoryUpdate::Allocation(_, _, _) => {}
            MemoryUpdate::Free(_, _, _) => {}
        }
    }
}