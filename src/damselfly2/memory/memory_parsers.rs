use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Read;
use std::iter::Peekable;
use std::str::Split;
use std::sync::Arc;
use addr2line::Context;
use owo_colors::OwoColorize;
use crate::damselfly::instruction::Instruction;
use crate::damselfly::memory_structs::{MemoryUpdate, RecordType};

#[derive(Default)]
pub struct MemorySysTraceParser {
    time: usize,
    record_queue: Vec<RecordType>,
    symbols: HashMap<usize, String>,
    prefix: String,
}

impl MemorySysTraceParser {
    pub fn new() -> MemorySysTraceParser {
        MemorySysTraceParser {
            time: 0, record_queue: Vec::new(), symbols: HashMap::new(), prefix: String::new()
        }
    }

    pub fn parse_log(&mut self, log: String, binary_path: String) -> Vec<Instruction> {
        let mut instructions = Vec::new();
        self.parse_symbols(&log, &binary_path);
        let mut log_iter = log.split('\n').peekable();
        while let Some(line) = log_iter.peek() {
            if Self::is_line_useless(line) {
                log_iter.next();
                continue;
            }
            println!("Processing instruction: {}", line.cyan());
            let instruction = self.process_instruction(&mut log_iter);
            instructions.push(instruction);
        }
        if !self.record_queue.is_empty() {
            let instruction = self.bake_instruction();
            instructions.push(instruction);
        }
        instructions
        // EOF
        /*
        let instruction = self.bake_instruction();
        self.instruction_tx.send(instruction).expect("[MemorySysTraceParser::parse_log]: Failed to send final instruction");
         */
    }

    fn is_line_useless(next_line: &str) -> bool {
        let split_line = next_line.split('>').collect::<Vec<_>>();
        if let Some(latter_half) = split_line.get(1) {
            let trimmed_string = latter_half.trim();
            if trimmed_string.starts_with('+') || trimmed_string.starts_with('-') || trimmed_string.starts_with('^') {
                return false;
            }
        }
        true
    }

    fn extract_addresses_from_log(log: &str) -> Vec<usize> {
        let mut set = HashSet::new();
        let log_iter = log.split('\n');
        for line in log_iter {
            if Self::is_line_useless(line) {
                continue;
            }
            if line.contains('^') {
                set.insert(Self::extract_trace_address(line));
            }
        }
        set
            .into_iter()
            .map(|address_string| {
                usize::from_str_radix(address_string.as_str(), 16)
                    .expect("[MemorySysTraceParser::extract_addresses_from_log]: Failed to cast address from String to u64")
            })
            .collect()
    }

    fn extract_trace_address(line: &str) -> String {
        let mut address = String::new();
        if let Some(open_bracket_pos) = line.rfind('[') {
            if let Some(close_bracket_pos) = line.rfind(']') {
                if close_bracket_pos > open_bracket_pos {
                    address = line[open_bracket_pos + 1..close_bracket_pos].to_string();
                }
            }
        }
        address
    }

    fn parse_symbols(&mut self, log: &str, binary_path: &str) {
        let addresses = Self::extract_addresses_from_log(log);
        let mut file = File::open(binary_path).unwrap();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();
        let object = object::File::parse(&*buffer).unwrap();
        let ctx = Context::new(&object).unwrap();

        let mut symbols = Vec::new();
        for address in &addresses {
            let mut symbol = String::new();
            let location = ctx.find_location(*address as u64).unwrap().unwrap();
            symbol.push_str(location.file.unwrap());
            symbol.push(':');
            symbol.push_str(location.line.unwrap().to_string().as_str());
            symbols.push(symbol);
        }

        self.prefix = Self::longest_common_prefix(&symbols);
        self.symbols = addresses.into_iter().zip(symbols).collect();
    }

    fn lookup_symbol(&self, query: String) -> Option<String> {
        let full_symbol = self.symbols.get(&usize::from_str_radix(query.as_str(), 16)
            .expect("[MemorySysTraceParser::lookup_symbol_str_short]: Failed to parse hex address string into usize"));
        full_symbol?;
        Some(full_symbol.unwrap()
            .trim_start_matches(&self.prefix)
            .to_string())
    }

    fn longest_common_prefix(strings: &Vec<String>) -> String {
        if strings.is_empty() {
            return String::new();
        }

        // Identify the shortest string in the vector
        let shortest = strings
            .iter()
            .filter(|string| string.starts_with('/'))
            .min_by_key(|s| s.len())
            .expect("[MemorySysTraceParser::longest_common_prefix]: Failed to identify shortest stacktrace path (log might be empty)");

        let mut prefix = String::new();
        for (i, char) in shortest.char_indices() {
            if strings.iter()
                .filter(|string| string.starts_with('/'))
                .all(|s| s.as_bytes()[i] == char as u8) {
                prefix.push(char);
            } else {
                break;
            }
        }

        prefix
    }

    pub fn process_instruction(&mut self, log_iter: &mut Peekable<Split<char>>) -> Instruction {
        let mut baked_instruction = None;
        for line in &mut *log_iter {
            if Self::is_line_useless(line) {
                continue;
            }
            let record = self.line_to_record(line)
                .expect("[MemorySysTraceParser::process_operation]: Failed to process line");
            match record {
                RecordType::StackTrace(_, _) => self.process_stacktrace(record),
                _ => { baked_instruction = self.process_alloc_or_free(Some(record)) },
            }
            if baked_instruction.is_some() { break; }
        }
        // EOF but last instruction left in queue
        if baked_instruction.is_none() && !self.record_queue.is_empty() {
            baked_instruction = Some(self.bake_instruction());
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
                        if *trace_address == alloc_address {
                            allocation_callstack.push_str(trace_callstack);
                            allocation_callstack.push('\n');
                        }
                    },
                    RecordType::Free(free_address, ref mut free_callstack) => {
                        // Check if we are tracing the correct address
                        if *trace_address == free_address {
                            //if !free_callstack.is_empty() { free_callstack.push('\n'); }
                            free_callstack.push_str(trace_callstack);
                            free_callstack.push('\n');
                        }
                    }
                    RecordType::StackTrace(_, _) => panic!("[MemorySysTraceParser::bake_instruction]: First instruction in instruction queue is a stacktrace, but it should be an alloc/free"),
                }
            }
        }

        let instruction;
        match first_rec {
            RecordType::Allocation(address, size, callstack) => {
                let memory_update = MemoryUpdate::Allocation(address, size, Arc::new(callstack));
                instruction = Instruction::new(self.time, memory_update);
                self.time += 1;
            },
            RecordType::Free(address, callstack) => {
                let memory_update = MemoryUpdate::Free(address, Arc::new(callstack));
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
            "^" => record = {
                let symbol = self.lookup_symbol(Self::extract_trace_address(split_dataline[2]))
                    .expect("[MemorySysTraceParser::parse_line]: Failed to lookup symbol");
                RecordType::StackTrace(0, symbol)
            },
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
