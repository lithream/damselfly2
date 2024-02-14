use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Read;
use std::iter::Peekable;
use std::str::Split;
use std::sync::Arc;
use addr2line::Context;
use owo_colors::OwoColorize;
use crate::damselfly2::memory::memory_update::{Allocation, Free, MemoryUpdate, MemoryUpdateType};

#[derive(Clone)]
pub enum RecordType {
    // (address, size, callstack)
    Allocation(usize, usize, String),
    // (address, callstack)
    Free(usize, String),
    // (address, callstack)
    StackTrace(usize, String),
}

#[derive(Default)]
pub struct MemorySysTraceParser {
    time: usize,
    record_queue: Vec<RecordType>,
    memory_updates: Vec<MemoryUpdateType>,
    symbols: HashMap<usize, String>,
    prefix: String,
}

impl MemorySysTraceParser {
    pub fn new() -> MemorySysTraceParser {
        MemorySysTraceParser {
            time: 0,
            record_queue: Vec::new(),
            memory_updates: Vec::new(),
            symbols: HashMap::new(),
            prefix: String::new()
        }
    }

    /// Parses a log file into a Vec of MemoryUpdateTypes, each containing an Allocation or a Free.
    ///
    /// # Arguments
    ///
    /// * `log`: log file as a String
    /// * `binary_path`: path to threadApp binary for debuginfo
    ///
    /// returns: a Vec of MemoryUpdateType (MemoryUpdate wrapped in an enum, ready for
    ///          interval overlap processing
    ///
    pub fn parse_log(mut self, log: String, binary_path: String) -> Vec<MemoryUpdateType> {
        self.parse_symbols(&log, &binary_path);
        let mut log_iter = log.split('\n').peekable();
        while let Some(line) = log_iter.peek() {
            if Self::is_line_useless(line) {
                log_iter.next();
                continue;
            }
            println!("Processing instruction: {}", line.cyan());
            let memory_update = self.process_instruction(&mut log_iter);
            self.memory_updates.push(memory_update);
        }
        if !self.record_queue.is_empty() {
            let memory_update = self.bake_memory_update();
            self.memory_updates.push(memory_update);
        }
        self.memory_updates
        // EOF
        /*
        let instruction = self.bake_instruction();
        self.instruction_tx.send(instruction).expect("[MemorySysTraceParser::parse_log]: Failed to send final instruction");
         */
    }

    /// Checks if a line in the log contains none of the following:
    /// Allocation information
    /// Free information
    /// Stacktrace information
    ///
    /// # Arguments
    ///
    /// * `line`: the line to check
    ///
    /// returns: true if useless, false if useful
    ///
    pub fn is_line_useless(line: &str) -> bool {
        let split_line = line.split('>').collect::<Vec<_>>();
        if let Some(latter_half) = split_line.get(1) {
            let trimmed_string = latter_half.trim();
            if trimmed_string.starts_with('+') || trimmed_string.starts_with('-') || trimmed_string.starts_with('^') {
                return false;
            }
        }
        true
    }

    /// Extracts all memory addresses from the log, ignoring lines that are deemed useless by
    /// is_line_useless.
    ///
    /// # Arguments
    ///
    /// * `log`: the entire log file
    ///
    /// returns: a Vec of every address in the log relevant to memory tracing

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

    /// Gets the memory address pointed to by a stacktrace line. This method does not check if the
    /// line is actually a stacktrace, so only use it when you are sure the line contains a stacktrace.
    ///
    /// # Arguments
    ///
    /// * `line`: a single line containing a stacktrace log
    ///
    /// returns: the address found in the line

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

    /// Finds symbols (FILENAME:LINE_NO) of addresses in a log. Also computes the longest prefix
    /// common to all symbols and stores it.
    ///
    /// # Arguments
    ///
    /// * `log`: the entire log
    /// * `binary_path`: path to the threadApp binary for debuginfo
    ///
    /// returns: nothing, as the longest prefix and symbols are stored as struct fields.

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

    /// Looks up the symbol corresponding to a hex address.
    ///
    /// # Arguments
    ///
    /// * `query`: the address in hex without a 0x prefix
    ///
    /// returns: the symbol if found, None otherwise

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

    pub fn process_instruction(&mut self, log_iter: &mut Peekable<Split<char>>) -> MemoryUpdateType {
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
            baked_instruction = Some(self.bake_memory_update());
        }
        baked_instruction.unwrap()
    }

    fn process_alloc_or_free(&mut self, record: Option<RecordType>) -> Option<MemoryUpdateType> {
        // If this is the first record in the log, push it into the queue and wait for StackTrace records
        if self.record_queue.is_empty() {
            self.record_queue.push(record.expect("[MemorySysTraceParser::process_alloc_or_free]: Queue is empty, but record is also None"));
            return None;
        }

        // Else, bake the previously stored alloc/free, clear the queue and push the latest record into it
        let baked_memory_update = self.bake_memory_update();
        self.record_queue.clear();
        if let Some(record) = record {
            self.record_queue.push(record);
        } else {
            // EOF. empty else block for clarity
        }
        Some(baked_memory_update)
    }

    fn bake_memory_update(&mut self) -> MemoryUpdateType {
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

        // Stack tracing complete, so we instantiate the MemoryUpdateType with the required data and return it
        let memory_update;
        match first_rec {
            RecordType::Allocation(address, size, callstack) => {
                memory_update = Allocation::new(address, size, Arc::new(callstack), self.time).wrap_in_enum();
                self.time += 1;
            },
            RecordType::Free(address, callstack) => {
                // We manually calculate the bytes to free, since the log file does not say how many bytes are freed
                let free_size = self.find_latest_allocation_size(address);
                if free_size.is_none() { panic!("[MSTParser::bake_instruction]: Can't find alloc for this free"); }
                memory_update = Free::new(address, free_size.unwrap(), Arc::new(callstack), self.time).wrap_in_enum();
                self.time += 1;
            }
            _ => { panic!("[MemorySysTraceParser::bake_instruction]: First instruction in instruction queue is a stacktrace, but it should be an alloc/free"); }
        }
        memory_update
    }

    fn find_latest_allocation_size(&self, address: usize) -> Option<usize> {
        for memory_update in self.memory_updates.iter().rev() {
            match memory_update {
                MemoryUpdateType::Allocation(allocation) => {
                    if allocation.get_absolute_address() == address {
                        return Some(allocation.get_absolute_size());
                    }
                }
                _ => {}
            }
        }
        None
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
