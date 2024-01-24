use std::collections::HashMap;
use std::fmt::Display;
use std::iter::Peekable;
use std::str::Split;
use std::sync::{mpsc};
use std::sync::mpsc::{Receiver, Sender};
use crate::damselfly_viewer::instruction::Instruction;

#[derive(PartialEq, Debug, Clone)]
pub enum MemoryUpdate {
    // (address, size, callstack)
    Allocation(usize, usize, String),
    // (address, callstack)
    Free(usize, String),
}

#[derive(Clone)]
pub enum RecordType {
    // (address, size, callstack)
    Allocation(usize, usize, String),
    // (address, callstack)
    Free(usize, String),
    // (address, callstack)
    StackTrace(usize, String),
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
            MemoryUpdate::Allocation(address, size, _) => format!("ALLOC: {} {}", address, size),
            MemoryUpdate::Free(address, callstack) => format!("FREE: {} {}", address, callstack),
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
    time: usize,
    record_queue: Vec<RecordType>,
}

impl MemorySysTraceParser {
    pub fn new() -> (MemorySysTraceParser, Receiver<Instruction>) {
        let (tx, rx) = mpsc::channel();
        (MemorySysTraceParser { instruction_tx: tx, map: HashMap::new(), time: 0, record_queue: Vec::new() }, rx)
    }

    pub fn parse_log(&mut self, log: String) {
        let mut log_iter = log.split('\n').peekable();
        while let Some(next_line) = log_iter.peek() {
            if self.is_line_useless(next_line) {
                log_iter.next();
                continue;
            }
            let instruction = self.process_instruction(&mut log_iter);
            self.instruction_tx.send(instruction).expect("[MemorySysTraceParser::parse_log]: Failed to send instruction");
        }
    }

    fn is_line_useless(&self, next_line: &&str) -> bool {
        let split_line = next_line.split('>').collect::<Vec<_>>();
        if let Some(latter_half) = split_line.get(1) {
            let trimmed_string = latter_half.trim();
            if trimmed_string.starts_with('+') || trimmed_string.starts_with('-') || trimmed_string.starts_with('^') {
                return false;
            }
        }
        true
    }

    pub fn process_instruction(&mut self, log_iter: &mut Peekable<Split<char>>) -> Instruction {
        let mut baked_instruction = None;
        while baked_instruction.is_none() {
            if let Some(line) = log_iter.next() {
                let record = self.line_to_record(line).expect("[MemorySysTraceParser::process_operation]: Failed to process line");
                match record {
                    RecordType::StackTrace(_, _) => self.process_stacktrace(record),
                    _ => baked_instruction = self.process_alloc_or_free(Some(record)),
                }
            } else {
                // EOF
                baked_instruction = self.process_alloc_or_free(None);
            }
        }
        baked_instruction.unwrap()
    }

    fn process_alloc_or_free(&mut self, record: Option<RecordType>) -> Option<Instruction> {
        // If this is the first record in the log, push it into the queue and wait for StackTrace records
        if self.record_queue.is_empty() {
            self.record_queue.push(record.expect("[MemorySysTraceParser::process_alloc_or_free]: Queue is empty, but record is also None"));
            return None;
        }

        // Else, bake the previously stored alloc/free, clear the queue and push the latest record into it
        let baked_instruction = self.bake_instruction();
        self.record_queue.clear();
        if let Some(record) = record {
            self.record_queue.push(record);
        } else {
            // EOF
        }
        Some(baked_instruction)
    }

    fn bake_instruction(&mut self) -> Instruction {
        let mut iter = self.record_queue.iter();
        let mut first_rec: RecordType = iter.next().expect("[MemorySysTraceParser::process_allocation]: Record queue empty").clone();
        for rec in iter {
            if let RecordType::StackTrace(trace_address, trace_callstack) = rec {
                match first_rec {
                    RecordType::Allocation(alloc_address, _, ref mut allocation_callstack) => {
                        // Check if we are tracing the correct address
                        if *trace_address != alloc_address { panic!("[MemorySysTraceParser::process_allocation]: Tracing wrong alloc"); }
                        if !allocation_callstack.is_empty() { allocation_callstack.push('\n'); }
                        allocation_callstack.push_str(trace_callstack);
                    },
                    RecordType::Free(free_address, ref mut free_callstack) => {
                        // Check if we are tracing the correct address
                        if *trace_address != free_address { panic!("[MemorySysTraceParser::process_allocation]: Tracing wrong free"); }
                        if !free_callstack.is_empty() { free_callstack.push('\n'); }
                        free_callstack.push_str(trace_callstack);
                    }
                    RecordType::StackTrace(_, _) => panic!("[MemorySysTraceParser::process_allocation]: First instruction in instruction queue is a stacktrace, but it should be an alloc/free"),
                }
            }
        }

        let instruction;
        match first_rec {
            RecordType::Allocation(address, size, callstack) => {
                let memory_update = MemoryUpdate::Allocation(address, size, callstack.clone());
                instruction = Instruction::new(self.time, memory_update);
                self.time += 1;
            },
            RecordType::Free(address, callstack) => {
                let memory_update = MemoryUpdate::Free(address, callstack.clone());
                instruction = Instruction::new(self.time, memory_update);
                self.time += 1;
            }
            _ => { panic!("[MemorySysTraceParser::bake_instruction]: First instruction in instruction queue is a stacktrace, but it should be an alloc/free"); }
        }
        instruction
    }

    fn process_stacktrace(&mut self, record: RecordType) {
        if self.record_queue.is_empty() {
            panic!("[MemorySysTraceParser::process_stacktrace]: First instruction in instruction queue cannot be a stacktrace");
        }
        self.record_queue.push(record);
    }

    fn line_to_record(&self, line: &str) -> Result<RecordType, &'static str> {
        let binding = line.split('>').collect::<Vec<_>>();
        let dataline = binding.get(1);
        if dataline.is_none() {
            return Err("[MemorySysTraceParser::parse_line]: Failed to split by > char");
        }
        let dataline = dataline.unwrap().trim();

        let split_dataline = dataline.split(' ').collect::<Vec<_>>();
        if split_dataline.len() < 2 || split_dataline.len() > 3 {
            return Err ("[MemorySysTraceParser::parse_line]: Line length mismatch");
        }

        let mut record;
        match split_dataline[0] {
            "+" => record = RecordType::Allocation(0, 0, String::new()),
            "-" => record = RecordType::Free(0, String::new()),
            "^" => record = RecordType::StackTrace(0, String::new()),
            _  => return Err("[MemorySysTraceParser::parse_line]: Invalid operation type"),
        }

        let address = usize::from_str_radix(split_dataline[1], 16)
            .expect("[MemorySysTraceParser::parse_line]: Failed to convert address to decimal");
        match record {
            RecordType::Allocation(ref mut default_address, ref mut default_size, _) => {
                *default_address = address;
                *default_size = split_dataline[2].parse()
                    .expect("[MemorySysTraceParser::parse_line]: Failed to read size");
            },
            RecordType::Free(ref mut default_address, _) => *default_address = address,
            RecordType::StackTrace(ref mut default_address, _) => *default_address = address,
        }

        Ok(record)
    }
}

#[cfg(test)]
mod tests {
    use crate::memory::{MemorySysTraceParser, MemoryUpdate, RecordType};

    #[test]
    fn is_line_useless_test() {
        let (memsystraceparser, _) = MemorySysTraceParser::new();
        let allocation_record = "00001068: 039dcb32 |V|A|005|        0 us   0003.677 s    < DT:0xE14DEEBC> + e150202c 14";
        let free_record = "00001053: 039dc9d7 |V|A|005|       67 us   0003.677 s    < DT:0xE14DEEBC> - e150202c";
        let stacktrace_record = "00001069: 039dcb32 |V|A|005|        0 us   0003.677 s    < DT:0xE14DEEBC> ^ e150202c [e045d83b]";
        let useless_record = "00000764: 036f855d |V|B|002|        0 us   0003.483 s    < DT:0xE1A5A75C> sched_switch from pid <0xe15ee200> (priority 0) to pid <0xe1a5a75c> (priority 144)";

        let log = [allocation_record, free_record, stacktrace_record, useless_record];
        let mut iter = log.iter().peekable();
        assert!(!memsystraceparser.is_line_useless(iter.peek().unwrap()));
        iter.next();
        assert!(!memsystraceparser.is_line_useless(iter.peek().unwrap()));
        iter.next();
        assert!(!memsystraceparser.is_line_useless(iter.peek().unwrap()));
        iter.next();
        assert!(memsystraceparser.is_line_useless(iter.peek().unwrap()));
    }

    #[test]
    fn bake_instruction_alloc_test() {
        let (mut memsystraceparser, _) = MemorySysTraceParser::new();
        memsystraceparser.record_queue.push(RecordType::Allocation(0, 4, "callstack".to_string()));
        memsystraceparser.record_queue.push(RecordType::StackTrace(0, "1".to_string()));
        memsystraceparser.record_queue.push(RecordType::StackTrace(0, "2".to_string()));
        memsystraceparser.record_queue.push(RecordType::StackTrace(0, "3".to_string()));
        let instruction = memsystraceparser.bake_instruction();
        assert!(matches!(instruction.get_operation(), MemoryUpdate::Allocation(_, _, _)));
        if let MemoryUpdate::Allocation(address, size, callstack) = instruction.get_operation() {
            assert_eq!(address, 0);
            assert_eq!(size, 4);
            assert_eq!(callstack, "callstack\n1\n2\n3");
        }
    }

    #[test]
    fn bake_instruction_free_test() {
        let (mut memsystraceparser, _) = MemorySysTraceParser::new();
        memsystraceparser.record_queue.push(RecordType::Free(0, "callstack".to_string()));
        memsystraceparser.record_queue.push(RecordType::StackTrace(0, "1".to_string()));
        memsystraceparser.record_queue.push(RecordType::StackTrace(0, "2".to_string()));
        memsystraceparser.record_queue.push(RecordType::StackTrace(0, "3".to_string()));
        let instruction = memsystraceparser.bake_instruction();
        assert!(matches!(instruction.get_operation(), MemoryUpdate::Free(_, _)));
        if let MemoryUpdate::Free(address, callstack) = instruction.get_operation() {
            assert_eq!(address, 0);
            assert_eq!(callstack, "callstack\n1\n2\n3");
        }
    }

    #[test]
    #[should_panic]
    fn bake_instruction_empty_test() {
        let (mut memsystraceparser, _) = MemorySysTraceParser::new();
        memsystraceparser.bake_instruction();
    }

    #[test]
    #[should_panic]
    fn bake_instruction_invalid_queue_trace_only_test() {
        let (mut memsystraceparser, _) = MemorySysTraceParser::new();
        memsystraceparser.record_queue.push(RecordType::StackTrace(0, "callstack".to_string()));
        memsystraceparser.bake_instruction();
    }

    #[test]
    #[should_panic]
    fn bake_instruction_invalid_queue_trace_first_allocation_test() {
        let (mut memsystraceparser, _) = MemorySysTraceParser::new();
        memsystraceparser.record_queue.push(RecordType::StackTrace(0, "callstack".to_string()));
        memsystraceparser.record_queue.push(RecordType::StackTrace(0, "callstack".to_string()));
        memsystraceparser.record_queue.push(RecordType::StackTrace(0, "callstack".to_string()));
        memsystraceparser.record_queue.push(RecordType::Allocation(0, 4, "callstack".to_string()));
        memsystraceparser.record_queue.push(RecordType::StackTrace(0, "callstack".to_string()));
        memsystraceparser.record_queue.push(RecordType::StackTrace(0, "callstack".to_string()));
        memsystraceparser.record_queue.push(RecordType::StackTrace(0, "callstack".to_string()));
        memsystraceparser.bake_instruction();
    }

    #[test]
    #[should_panic]
    fn bake_instruction_invalid_queue_trace_first_free_test() {
        let (mut memsystraceparser, _) = MemorySysTraceParser::new();
        memsystraceparser.record_queue.push(RecordType::StackTrace(0, "callstack".to_string()));
        memsystraceparser.record_queue.push(RecordType::StackTrace(0, "callstack".to_string()));
        memsystraceparser.record_queue.push(RecordType::StackTrace(0, "callstack".to_string()));
        memsystraceparser.record_queue.push(RecordType::Free(0, "callstack".to_string()));
        memsystraceparser.record_queue.push(RecordType::StackTrace(0, "callstack".to_string()));
        memsystraceparser.record_queue.push(RecordType::StackTrace(0, "callstack".to_string()));
        memsystraceparser.record_queue.push(RecordType::StackTrace(0, "callstack".to_string()));
        memsystraceparser.bake_instruction();
    }

    #[test]
    fn process_alloc_or_free_first_record_test(){
        let (mut memsystraceparser, _) = MemorySysTraceParser::new();
        let record = RecordType::Allocation(0, 4, "callstack".to_string());
        let instruction = memsystraceparser.process_alloc_or_free(Some(record));
        assert!(instruction.is_none());
        assert_eq!(memsystraceparser.record_queue.len(), 1);
        match memsystraceparser.record_queue.first().unwrap() {
            RecordType::Allocation(address, size, callstack) => {
                assert_eq!(*address, 0);
                assert_eq!(*size, 4);
                assert_eq!(*callstack, "callstack".to_string());
            }
            RecordType::Free(_, _) => panic!("Wrong type: Free"),
            RecordType::StackTrace(_, _) => panic!("Wrong type: Stacktrace"),
        }
    }

    #[test]
    fn process_alloc_or_free_existing_records_test() {
        let (mut memsystraceparser, _) = MemorySysTraceParser::new();
        let alloc_record = RecordType::Allocation(0, 4, "callstack".to_string());
        let records = vec![
            RecordType::StackTrace(0, "1".to_string()),
            RecordType::StackTrace(0, "2".to_string()),
            RecordType::StackTrace(0, "3".to_string()),
        ];

        memsystraceparser.process_alloc_or_free(Some(alloc_record));
        for record in records {
            memsystraceparser.process_stacktrace(record);
        }

        // Current queue status
        // | Alloc0 | Trace1 | Trace2 | Trace3 |
        let instruction = memsystraceparser.process_alloc_or_free(
            Some(RecordType::Allocation(4, 4, "callstack2".to_string()))
        ).unwrap();
        // | Alloc4 |
        // instruction = Alloc0 with Trace 1-3

        match instruction.get_operation() {
            MemoryUpdate::Allocation(address, size, callstack) => {
                assert_eq!(address, 0);
                assert_eq!(size, 4);
                assert_eq!(callstack, "callstack\n1\n2\n3");
            }
            MemoryUpdate::Free(_, _) => panic!("Wrong type: Free"),
        }

        let records = vec![
            RecordType::StackTrace(4, "4".to_string()),
            RecordType::StackTrace(4, "5".to_string()),
            RecordType::StackTrace(4, "6".to_string()),
        ];

        for record in records {
            memsystraceparser.process_stacktrace(record);
        }

        // | Alloc4 | Trace4 | Trace5 | Trace6 |
        let instruction = memsystraceparser.process_alloc_or_free(
            Some(RecordType::Free(0, "callstack3".to_string()))
        ).unwrap();
        // | Free0 |
        // instruction = Alloc4 with Trace 1-3

        match instruction.get_operation() {
            MemoryUpdate::Allocation(address, size, callstack) => {
                assert_eq!(address, 4);
                assert_eq!(size, 4);
                assert_eq!(callstack, "callstack2\n4\n5\n6");
            }
            MemoryUpdate::Free(_, _) => panic!("Wrong type: Free"),
        }

        // EOF
        let instruction = memsystraceparser.process_alloc_or_free(None).unwrap();
        // Empty
        // instruction = Free

        match instruction.get_operation() {
            MemoryUpdate::Allocation(_, _, _) => panic!("Wrong type: Allocation"),
            MemoryUpdate::Free(address, callstack) => {
                assert_eq!(address, 0);
                assert_eq!(callstack, "callstack3");
            }
        }
    }

    #[test]
    fn line_to_record_alloc_test() {
        let (memsystraceparser, _) = MemorySysTraceParser::new();
        let line = "00001444: 039e0edc |V|A|005|        0 us   0003.678 s    < DT:0xE1504C74> + e150206c 20";
        let record = memsystraceparser.line_to_record(line).unwrap();
        match record {
            RecordType::Allocation(address, size, callstack) => {
                assert_eq!(address, 3780124780);
                assert_eq!(size, 20);
                assert!(callstack.is_empty());
            }
            RecordType::Free(_, _) => panic!("Wrong record type: Free"),
            RecordType::StackTrace(_, _) => panic!("Wrong record type: Stacktrace"),
        }
    }
}
/*
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

 */