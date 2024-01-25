use std::collections::HashMap;
use std::fmt::Display;
use std::iter::Peekable;
use std::str::Split;
use std::sync::{mpsc};
use std::sync::mpsc::{Receiver, Sender};
use crate::damselfly_viewer::consts::DEFAULT_BLOCK_SIZE;
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
    // parent block, callstack
    Allocated(usize, String),
    // parent block, callstack
    PartiallyAllocated(usize, String),
    Free(String),
}

impl Display for MemoryUpdate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            MemoryUpdate::Allocation(address, size, _) => format!("ALLOC: {:x} {}", address, size),
            MemoryUpdate::Free(address, callstack) => format!("FREE: {:x} {}", address, callstack),
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

// CALLSTACK TRACING NOT IMPLEMENTED YET!!! TODO!
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
        let mut log_iter = log.split('\n');
        while let Some(line) = log_iter.next() {
            if self.is_line_useless(line) {
                continue;
            }
            let instruction = self.process_instruction(&mut log_iter);
            self.instruction_tx.send(instruction).expect("[MemorySysTraceParser::parse_log]: Failed to send instruction");
        }
        // EOF
    }

    fn is_line_useless(&self, next_line: &str) -> bool {
        let split_line = next_line.split('>').collect::<Vec<_>>();
        if let Some(latter_half) = split_line.get(1) {
            let trimmed_string = latter_half.trim();
            if trimmed_string.starts_with('+') || trimmed_string.starts_with('-') || trimmed_string.starts_with('^') {
                return false;
            }
        }
        true
    }

    pub fn process_instruction(&mut self, log_iter: &mut Split<char>) -> Instruction {
        let mut baked_instruction = None;
        while baked_instruction.is_none() {
            for line in &mut *log_iter {
                if self.is_line_useless(line) {
                    continue;
                }
                let record = self.line_to_record(line).expect("[MemorySysTraceParser::process_operation]: Failed to process line");
                match record {
                    RecordType::StackTrace(_, _) => self.process_stacktrace(record),
                    _ => baked_instruction = self.process_alloc_or_free(Some(record)),
                }
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
        let mut first_rec: RecordType = iter.next().expect("[MemorySysTraceParser::bake_instruction]: Record queue empty").clone();
        for rec in iter {
            if let RecordType::StackTrace(trace_address, trace_callstack) = rec {
                match first_rec {
                    RecordType::Allocation(alloc_address, _, ref mut allocation_callstack) => {
                        // Check if we are tracing the correct address
                        if *trace_address != alloc_address { panic!("[MemorySysTraceParser::bake_instruction]: Tracing wrong alloc"); }
                        if !allocation_callstack.is_empty() { allocation_callstack.push('\n'); }
                        allocation_callstack.push_str(trace_callstack);
                    },
                    RecordType::Free(free_address, ref mut free_callstack) => {
                        // Check if we are tracing the correct address
                        if *trace_address != free_address { panic!("[MemorySysTraceParser::bake_instruction]: Tracing wrong free"); }
                        if !free_callstack.is_empty() { free_callstack.push('\n'); }
                        free_callstack.push_str(trace_callstack);
                    }
                    RecordType::StackTrace(_, _) => panic!("[MemorySysTraceParser::bake_instruction]: First instruction in instruction queue is a stacktrace, but it should be an alloc/free"),
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
                *default_size = usize::from_str_radix(split_dataline[2], 16)
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
    use std::arch::x86_64::_mm256_abs_epi8;
    use crate::memory::{MemorySysTraceParser, MemoryUpdate, RecordType};

    #[test]
    fn is_line_useless_test() {
        let (mst_parser, _) = MemorySysTraceParser::new();
        let allocation_record = "00001068: 039dcb32 |V|A|005|        0 us   0003.677 s    < DT:0xE14DEEBC> + e150202c 14";
        let free_record = "00001053: 039dc9d7 |V|A|005|       67 us   0003.677 s    < DT:0xE14DEEBC> - e150202c";
        let stacktrace_record = "00001069: 039dcb32 |V|A|005|        0 us   0003.677 s    < DT:0xE14DEEBC> ^ e150202c [e045d83b]";
        let useless_record = "00000764: 036f855d |V|B|002|        0 us   0003.483 s    < DT:0xE1A5A75C> sched_switch from pid <0xe15ee200> (priority 0) to pid <0xe1a5a75c> (priority 144)";

        let log = [allocation_record, free_record, stacktrace_record, useless_record];
        let mut iter = log.iter().peekable();
        assert!(!mst_parser.is_line_useless(iter.peek().unwrap()));
        iter.next();
        assert!(!mst_parser.is_line_useless(iter.peek().unwrap()));
        iter.next();
        assert!(!mst_parser.is_line_useless(iter.peek().unwrap()));
        iter.next();
        assert!(mst_parser.is_line_useless(iter.peek().unwrap()));
    }

    #[test]
    fn bake_instruction_alloc_test() {
        let (mut mst_parser, _) = MemorySysTraceParser::new();
        mst_parser.record_queue.push(RecordType::Allocation(0, 4, "callstack".to_string()));
        mst_parser.record_queue.push(RecordType::StackTrace(0, "1".to_string()));
        mst_parser.record_queue.push(RecordType::StackTrace(0, "2".to_string()));
        mst_parser.record_queue.push(RecordType::StackTrace(0, "3".to_string()));
        let instruction = mst_parser.bake_instruction();
        assert!(matches!(instruction.get_operation(), MemoryUpdate::Allocation(_, _, _)));
        if let MemoryUpdate::Allocation(address, size, callstack) = instruction.get_operation() {
            assert_eq!(address, 0);
            assert_eq!(size, 4);
            assert_eq!(callstack, "callstack\n1\n2\n3");
        }
    }

    #[test]
    fn bake_instruction_free_test() {
        let (mut mst_parser, _) = MemorySysTraceParser::new();
        mst_parser.record_queue.push(RecordType::Free(0, "callstack".to_string()));
        mst_parser.record_queue.push(RecordType::StackTrace(0, "1".to_string()));
        mst_parser.record_queue.push(RecordType::StackTrace(0, "2".to_string()));
        mst_parser.record_queue.push(RecordType::StackTrace(0, "3".to_string()));
        let instruction = mst_parser.bake_instruction();
        assert!(matches!(instruction.get_operation(), MemoryUpdate::Free(_, _)));
        if let MemoryUpdate::Free(address, callstack) = instruction.get_operation() {
            assert_eq!(address, 0);
            assert_eq!(callstack, "callstack\n1\n2\n3");
        }
    }

    #[test]
    #[should_panic]
    fn bake_instruction_empty_test() {
        let (mut mst_parser, _) = MemorySysTraceParser::new();
        mst_parser.bake_instruction();
    }

    #[test]
    #[should_panic]
    fn bake_instruction_invalid_queue_trace_only_test() {
        let (mut mst_parser, _) = MemorySysTraceParser::new();
        mst_parser.record_queue.push(RecordType::StackTrace(0, "callstack".to_string()));
        mst_parser.bake_instruction();
    }

    #[test]
    #[should_panic]
    fn bake_instruction_invalid_queue_trace_first_allocation_test() {
        let (mut mst_parser, _) = MemorySysTraceParser::new();
        mst_parser.record_queue.push(RecordType::StackTrace(0, "callstack".to_string()));
        mst_parser.record_queue.push(RecordType::StackTrace(0, "callstack".to_string()));
        mst_parser.record_queue.push(RecordType::StackTrace(0, "callstack".to_string()));
        mst_parser.record_queue.push(RecordType::Allocation(0, 4, "callstack".to_string()));
        mst_parser.record_queue.push(RecordType::StackTrace(0, "callstack".to_string()));
        mst_parser.record_queue.push(RecordType::StackTrace(0, "callstack".to_string()));
        mst_parser.record_queue.push(RecordType::StackTrace(0, "callstack".to_string()));
        mst_parser.bake_instruction();
    }

    #[test]
    #[should_panic]
    fn bake_instruction_invalid_queue_trace_first_free_test() {
        let (mut mst_parser, _) = MemorySysTraceParser::new();
        mst_parser.record_queue.push(RecordType::StackTrace(0, "callstack".to_string()));
        mst_parser.record_queue.push(RecordType::StackTrace(0, "callstack".to_string()));
        mst_parser.record_queue.push(RecordType::StackTrace(0, "callstack".to_string()));
        mst_parser.record_queue.push(RecordType::Free(0, "callstack".to_string()));
        mst_parser.record_queue.push(RecordType::StackTrace(0, "callstack".to_string()));
        mst_parser.record_queue.push(RecordType::StackTrace(0, "callstack".to_string()));
        mst_parser.record_queue.push(RecordType::StackTrace(0, "callstack".to_string()));
        mst_parser.bake_instruction();
    }

    #[test]
    fn process_alloc_or_free_first_record_test(){
        let (mut mst_parser, _) = MemorySysTraceParser::new();
        let record = RecordType::Allocation(0, 4, "callstack".to_string());
        let instruction = mst_parser.process_alloc_or_free(Some(record));
        assert!(instruction.is_none());
        assert_eq!(mst_parser.record_queue.len(), 1);
        match mst_parser.record_queue.first().unwrap() {
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
        let (mut mst_parser, _) = MemorySysTraceParser::new();
        let alloc_record = RecordType::Allocation(0, 4, "callstack".to_string());
        let records = vec![
            RecordType::StackTrace(0, "1".to_string()),
            RecordType::StackTrace(0, "2".to_string()),
            RecordType::StackTrace(0, "3".to_string()),
        ];

        mst_parser.process_alloc_or_free(Some(alloc_record));
        for record in records {
            mst_parser.process_stacktrace(record);
        }

        // Current queue status
        // | Alloc0 | Trace1 | Trace2 | Trace3 |
        let instruction = mst_parser.process_alloc_or_free(
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
            mst_parser.process_stacktrace(record);
        }

        // | Alloc4 | Trace4 | Trace5 | Trace6 |
        let instruction = mst_parser.process_alloc_or_free(
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
        let instruction = mst_parser.process_alloc_or_free(None).unwrap();
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
    #[should_panic]
    fn process_stacktrace_empty_queue_test() {
        let (mut mst_parser, _) = MemorySysTraceParser::new();
        mst_parser.process_stacktrace(
            RecordType::StackTrace(0, "1".to_string())
        );
    }

    #[test]
    fn line_to_record_alloc_test() {
        let (mst_parser, _) = MemorySysTraceParser::new();
        let line = "00001444: 039e0edc |V|A|005|        0 us   0003.678 s    < DT:0xE1504C74> + e150206c 20";
        let record = mst_parser.line_to_record(line).unwrap();
        match record {
            RecordType::Allocation(address, size, callstack) => {
                assert_eq!(address, 3780124780);
                assert_eq!(size, 32);
                assert!(callstack.is_empty());
            }
            RecordType::Free(_, _) => panic!("Wrong record type: Free"),
            RecordType::StackTrace(_, _) => panic!("Wrong record type: Stacktrace"),
        }
    }

    #[test]
    fn line_to_record_free_test() {
        let (mst_parser, _) = MemorySysTraceParser::new();
        let line = "00001190: 039dd8f5 |V|A|005|       13 us   0003.677 s    < DT:0xE1504B54> - e150202c";
        let record = mst_parser.line_to_record(line).unwrap();
        match record {
            RecordType::Allocation(_, _, _) => panic!("Wrong type: Allocation"),
            RecordType::Free(address, callstack) => {
                assert_eq!(address, 3780124716);
                assert!(callstack.is_empty());
            }
            RecordType::StackTrace(_, _) => panic!("Wrong type: Stacktrace"),
        }
    }

    #[test]
    fn line_to_record_trace_test() {
        let (mst_parser, _) = MemorySysTraceParser::new();
        let line = "00001191: 039dd8f5 |V|A|005|        0 us   0003.677 s    < DT:0xE1504B54> ^ e150202c [e045d889]";
        let record = mst_parser.line_to_record(line).unwrap();
        match record {
            RecordType::Allocation(_, _, _) => panic!("Wrong type: Allocation"),
            RecordType::Free(_, _) => panic!("Wrong type: Free"),
            RecordType::StackTrace(address, callstack) => {
                assert_eq!(address, 3780124716);
            }
        }
    }

    #[test]
    fn parse_log_test() {
        let (mut mst_parser, instruction_rx) = MemorySysTraceParser::new();
        let log = "\
       00001066: 039dcad2 |V|B|002|        0 us   0003.677 s    < DT:0xE14DEEBC> ActivityMonitorStandard::runTimer::state 2
00001067: 039dcb32 |V|B|002|        6 us   0003.677 s    < DT:0xE14DEEBC> ActivityMonitorStandard::runTimer: starting timer to state 2 because of no state activity
00001068: 039dcb32 |V|A|005|        0 us   0003.677 s    < DT:0xE14DEEBC> + e150202c 14
00001069: 039dcb32 |V|A|005|        0 us   0003.677 s    < DT:0xE14DEEBC> ^ e150202c [e045d83b]
00001070: 039dcb41 |V|A|005|        0 us   0003.677 s    < DT:0xE14DEEBC> ^ e150202c [e045f0eb]
00001071: 039dcb41 |V|A|005|        0 us   0003.677 s    < DT:0xE14DEEBC> ^ e150202c [e0015de9]
00001072: 039dcb41 |V|A|005|        0 us   0003.677 s    < DT:0xE14DEEBC> ^ e150202c [e0016197]
00001073: 039dcb4c |V|A|005|        0 us   0003.677 s    < DT:0xE14DEEBC> ^ e150202c [e001a835]
00001074: 039dcb4c |V|A|005|        0 us   0003.677 s    < DT:0xE14DEEBC> ^ e150202c [e001a935]
00001075: 039dcb4c |V|A|005|        0 us   0003.677 s    < DT:0xE14DEEBC> ^ e150202c [e001a9d1]
00001076: 039dcb62 |V|A|005|        1 us   0003.677 s    < DT:0xE14DEEBC> ^ e150202c [e001b37b]
00001077: 039dcb62 |V|A|005|        0 us   0003.677 s    < DT:0xE14DEEBC> ^ e150202c [e001b699]
00001078: 039dcb70 |V|A|005|        0 us   0003.677 s    < DT:0xE14DEEBC> ^ e150202c [e045506d]
00001079: 039dcb70 |V|A|005|        0 us   0003.677 s    < DT:0xE14DEEBC> ^ e150202c [e00146eb]
00001080: 039dcb70 |V|A|005|        0 us   0003.677 s    < DT:0xE14DEEBC> ^ e150202c [e001b6ad]
00001081: 039dcb77 |V|A|005|        0 us   0003.677 s    < DT:0xE14DEEBC> ^ e150202c [e00145a9]
00001082: 039dcb77 |V|A|005|        0 us   0003.677 s    < DT:0xE14DEEBC> ^ e150202c [e00115e1]
00001083: 039dcb77 |V|A|005|        0 us   0003.677 s    < DT:0xE14DEEBC> ^ e150202c [e0011a2d]
00001084: 039dcca1 |V|A|005|       19 us   0003.677 s    < DT:0xE14DEEBC> ^ e150202c [e048c81f]
00001085: 039dcca1 |V|A|005|        0 us   0003.677 s    < DT:0xE14DEEBC> ^ e150202c [e04865ef]
00001086: 039dcca1 |V|A|002|        0 us   0003.677 s    < DT:0xE14DEEBC> SSC::handleActivityStateInProgressEvent state 3
00001087: 039dcdad |V|A|005|       17 us   0003.677 s    < DT:0xE14DEEBC> - e150202c
00001088: 039dcdad |V|A|005|        0 us   0003.677 s    < DT:0xE14DEEBC> ^ e150202c [e045d889]
00001089: 039dcdad |V|A|005|        0 us   0003.677 s    < DT:0xE14DEEBC> ^ e150202c [e001ab0b]
00001090: 039dcdb7 |V|A|005|        0 us   0003.677 s    < DT:0xE14DEEBC> ^ e150202c [e001ac83]
00001091: 039dcdb7 |V|A|005|        0 us   0003.677 s    < DT:0xE14DEEBC> ^ e150202c [e001b4ed]
00001092: 039dcdc4 |V|A|005|        0 us   0003.677 s    < DT:0xE14DEEBC> ^ e150202c [e001b577]
00001093: 039dcdc4 |V|A|005|        0 us   0003.677 s    < DT:0xE14DEEBC> ^ e150202c [e001b6ad]
00001094: 039dcdc4 |V|A|005|        0 us   0003.677 s    < DT:0xE14DEEBC> ^ e150202c [e00146eb]
00001095: 039dcdce |V|A|005|        0 us   0003.677 s    < DT:0xE14DEEBC> ^ e150202c [e001b6ad]
00001096: 039dcdce |V|A|005|        0 us   0003.677 s    < DT:0xE14DEEBC> ^ e150202c [e00145a9]
00001097: 039dcdce |V|A|005|        0 us   0003.677 s    < DT:0xE14DEEBC> ^ e150202c [e00115e1]
00001098: 039dce14 |V|A|005|        4 us   0003.677 s    < DT:0xE14DEEBC> ^ e150202c [e0011a2d]
00001099: 039dce14 |V|A|005|        0 us   0003.677 s    < DT:0xE14DEEBC> ^ e150202c [e048c81f]
00001100: 039dce14 |V|A|005|        0 us   0003.677 s    < DT:0xE14DEEBC> ^ e150202c [e04865ef]
00001101: 039dcf14 |V|B|002|       16 us   0003.677 s    < DT:0xE14DEEBC> ActivityMonitorStandard::runTimer: notified InProgress state 3 event
00001102: 039dcff1 |V|B|002|       14 us   0003.677 s    < DT:0xE1504B54> sched_switch from pid <0xe14ca764> (priority 252) to pid <0xe1504b54> (priority 253)
00001103: 039dcff1 |V|A|002|        0 us   0003.677 s    < DT:0xE1504B54> SSC::StateSchedulerController Async path for scheduling power level change from: 0 to: 2
00001104: 039dd04f |V|A|002|        6 us   0003.677 s    < DT:0xE1504B54> SSC::[StateSchedulerController] scheduling Power Level Change from: 0 to: 2
00001105: 039dd04f |V|A|005|        0 us   0003.677 s    < DT:0xE1504B54> + e15020a4 6c
 ";

        mst_parser.parse_log(log.to_string());
        let alloc = instruction_rx.recv().unwrap();
        if let MemoryUpdate::Allocation(address, size, callstack) = alloc.get_operation() {
            assert_eq!(address, 3780124716);
            assert_eq!(size, 20);
            assert!(callstack.is_empty());
        }
        assert_eq!(alloc.get_timestamp(), 0);
        let free = instruction_rx.recv().unwrap();
        if let MemoryUpdate::Free(address, callstack) = free.get_operation() {
            assert_eq!(address, 3780124716);
            assert!(callstack.is_empty());
        }
        assert_eq!(free.get_timestamp(), 1);
        let alloc = instruction_rx.recv().unwrap();
        if let MemoryUpdate::Allocation(address, size, callstack) = alloc.get_operation() {
            assert_eq!(address, 3780124836);
            assert_eq!(size, 108);
            assert!(callstack.is_empty());
        }
        assert_eq!(alloc.get_timestamp(), 2);
    }

    #[test]
    fn parse_log_garbage_at_end_test() {
        let (mut mst_parser, instruction_rx) = MemorySysTraceParser::new();
        let log = "\
       00057601: 0b1972d8 |V|A|005|       11 us   0011.712 s    < DT:0xE14B3B4C> - e1504dc4
00057602: 0b1972d8 |V|A|005|        0 us   0011.712 s    < DT:0xE14B3B4C> ^ e1504dc4 [e045d889]
00057603: 0b1972d8 |V|A|005|        0 us   0011.712 s    < DT:0xE14B3B4C> ^ e1504dc4 [e048c88f]
00057604: 0b19731e |V|A|005|        4 us   0011.712 s    < DT:0xE14B3B4C> ^ e1504dc4 [e048c81f]
00057605: 0b19731e |V|A|005|        0 us   0011.712 s    < DT:0xE14B3B4C> ^ e1504dc4 [e04865ef]
00057606: 0b19741f |V|B|002|       16 us   0011.712 s    < DT:0xE14DEEBC> sched_switch from pid <0xe14b3b4c> (priority 252) to pid <0xe14deebc> (priority 255)
00057607: 0b197a34 |V|B|002|       99 us   0011.712 s    < DT:0xE14E6D94> sched_switch from pid <0xe14deebc> (priority 255) to pid <0xe14e6d94> (priority 235)
00057608: 0b197a34 |V|B|002|        0 us   0011.712 s    < DT:0xE14DEEBC> sched_switch from pid <0xe14e6d94> (priority 235) to pid <0xe14deebc> (priority 235)
00057609: 0b197a70 |V|B|002|        3 us   0011.712 s    < DT:0xE14E6D94> sched_switch from pid <0xe14deebc> (priority 255) to pid <0xe14e6d94> (priority 235)
 ";
        mst_parser.parse_log(log.to_string());
        let free = instruction_rx.recv().unwrap();
    }
}