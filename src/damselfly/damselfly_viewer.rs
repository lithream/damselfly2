use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;
use log::debug;
use crate::damselfly::Damselfly;
use crate::memory::{MemorySnapshot, MemoryStatus};


const DEFAULT_TIMESPAN: u64 = 25;
const DEFAULT_MEMORYSPAN: u64 = 1024;

#[derive(Debug, Default, Clone)]
pub struct MemoryUsage {
    pub memory_used_percentage: f64,
    pub memory_used_absolute: f64,
    pub total_memory: u64
}

#[derive(Debug)]
pub struct DamselflyViewer {
    snapshot_rx: mpsc::Receiver<MemorySnapshot>,
    timespan: (u64, u64),
    timespan_is_unlocked: bool,
    memoryspan: (u64, u64),
    memoryspan_is_unlocked: bool,
    memory_usage_snapshots: Vec<MemoryUsage>,
    memory_map_snapshots: Vec<HashMap<u64, MemoryStatus>>
}

impl DamselflyViewer {
    pub fn new(mut damselfly: Damselfly, snapshot_rx: mpsc::Receiver<MemorySnapshot>) -> DamselflyViewer {
        thread::spawn(move || {
            loop {
                damselfly.execute_instruction();
            }
        });
        DamselflyViewer {
            snapshot_rx,
            timespan: (0, DEFAULT_TIMESPAN),
            timespan_is_unlocked: false,
            memoryspan: (0, DEFAULT_MEMORYSPAN),
            memoryspan_is_unlocked: false,
            memory_usage_snapshots: Vec::new(),
            memory_map_snapshots: Vec::new(),
        }
    }

    pub fn shift_span(&mut self, mut left: u64, mut right: u64, units: i64) -> (u64, u64) {
        debug_assert!(right > left);
        let span = right - left;
        let absolute_shift = units * span as i64;

        left = left.saturating_add_signed(absolute_shift).clamp(u64::MIN, right + absolute_shift.unsigned_abs());
        right = right.saturating_add_signed(absolute_shift).clamp(u64::MIN + span, u64::MAX);
        (left, right)
    }

    pub fn shift_timespan(&mut self, units: i64) {
        self.timespan_is_unlocked = true;
        (self.timespan.0, self.timespan.1) = self.shift_span(self.timespan.0, self.timespan.1, units);
    }

    pub fn lock_timespan(&mut self) {
        let current_span = self.timespan.1 - self.timespan.0;
        self.timespan.1 = (self.memory_usage_snapshots.len().saturating_sub(1)) as u64;
        self.timespan.0 = self.timespan.1.saturating_sub(current_span);
        self.timespan_is_unlocked = false;
    }

    pub fn shift_memoryspan(&mut self, units: i64) {
        self.memoryspan_is_unlocked = true;
        (self.memoryspan.0, self.memoryspan.1) = self.shift_span(self.memoryspan.0, self.memoryspan.1, units);
    }

    pub fn lock_memoryspan(&mut self) {
        self.memoryspan_is_unlocked = false;
    }

    pub fn update(&mut self) {
        let update = self.snapshot_rx.recv();
        match update {
            Ok(snapshot) => self.parse_snapshot(snapshot),
            Err(_) => debug!("[DamselflyViewer::update]: Snapshot channel hung up.")
        }

        if !self.timespan_is_unlocked {
            self.timespan.1 += 1;
            self.timespan.0 += 1;
        }

        if !self.memoryspan_is_unlocked {
            // do nothing, no memoryspan locking for now
        }
    }

    pub fn parse_snapshot(&mut self, snapshot: MemorySnapshot) {
        let memory_usage = MemoryUsage {
            memory_used_percentage: snapshot.memory_usage.0 / snapshot.memory_usage.1 as f64,
            memory_used_absolute: snapshot.memory_usage.0,
            total_memory: snapshot.memory_usage.1
        };
        self.memory_usage_snapshots.push(memory_usage);
        self.memory_map_snapshots.push(snapshot.memory_map);
    }

    pub fn get_memory_usage(&self) -> MemoryUsage {
        let memory_usage = self.memory_usage_snapshots.last();
        match memory_usage {
            None => {
                MemoryUsage::default()
            }
            Some(memory_usage) => (*memory_usage).clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use log::debug;
    use crate::damselfly::Damselfly;
    use crate::damselfly::damselfly_viewer::DamselflyViewer;
    use crate::memory::{MemoryStatus, MemoryStub, MemoryUpdate};

    fn initialise_viewer() -> (DamselflyViewer, MemoryStub) {
        let (memory_stub, instruction_rx) = MemoryStub::new();
        let (damselfly, snapshot_rx) = Damselfly::new(instruction_rx);
        let damselfly_viewer = DamselflyViewer::new(damselfly, snapshot_rx);
        (damselfly_viewer, memory_stub)
    }

    #[test]
    fn shift_timespan() {
        let (mut damselfly_viewer, _memory_stub) = initialise_viewer();
        damselfly_viewer.timespan.0 = 50;
        damselfly_viewer.timespan.1 = 75;
        assert_eq!(damselfly_viewer.timespan.0, 50);
        assert_eq!(damselfly_viewer.timespan.1, 75);
        damselfly_viewer.shift_timespan(-1);
        assert_eq!(damselfly_viewer.timespan.0, 25);
        assert_eq!(damselfly_viewer.timespan.1, 50);
        damselfly_viewer.shift_timespan(1);
        assert_eq!(damselfly_viewer.timespan.0, 50);
        assert_eq!(damselfly_viewer.timespan.1, 75);
    }

    #[test]
    fn shift_timespan_left_cap() {
        let (mut damselfly_viewer, _memory_stub) = initialise_viewer();
        damselfly_viewer.timespan.0 = 50;
        damselfly_viewer.timespan.1 = 75;
        damselfly_viewer.shift_timespan(-3);
        assert_eq!(damselfly_viewer.timespan.0, 0);
        assert_eq!(damselfly_viewer.timespan.1, 25);
        damselfly_viewer.shift_timespan(1);
        assert_eq!(damselfly_viewer.timespan.0, 25);
        assert_eq!(damselfly_viewer.timespan.1, 50);
        damselfly_viewer.shift_timespan(2);
        assert_eq!(damselfly_viewer.timespan.0, 75);
        assert_eq!(damselfly_viewer.timespan.1, 100);
    }

    #[test]
    fn shift_timespan_right() {
        let (mut damselfly_viewer, _memory_stub) = initialise_viewer();
        damselfly_viewer.timespan.0 = 50;
        damselfly_viewer.timespan.1 = 75;
        damselfly_viewer.shift_timespan(1);
        assert_eq!(damselfly_viewer.timespan.0, 75);
        assert_eq!(damselfly_viewer.timespan.1, 100);
        damselfly_viewer.shift_timespan(2);
        assert_eq!(damselfly_viewer.timespan.0, 125);
        assert_eq!(damselfly_viewer.timespan.1, 150);
    }

    #[test]
    fn shift_memoryspan() {
        let (mut damselfly_viewer, _memory_stub) = initialise_viewer();
        damselfly_viewer.memoryspan.0 = 0;
        damselfly_viewer.memoryspan.1 = 1024;
        damselfly_viewer.shift_memoryspan(1);
        assert_eq!(damselfly_viewer.memoryspan.0, 1024);
        assert_eq!(damselfly_viewer.memoryspan.1, 2048);
        damselfly_viewer.shift_memoryspan(-1);
        assert_eq!(damselfly_viewer.memoryspan.0, 0);
        assert_eq!(damselfly_viewer.memoryspan.1, 1024);
    }

    #[test]
    fn shift_memoryspan_left_cap() {
        let (mut damselfly_viewer, _memory_stub) = initialise_viewer();
        damselfly_viewer.memoryspan.0 = 0;
        damselfly_viewer.memoryspan.1 = 1024;
        damselfly_viewer.shift_memoryspan(-3);
        assert_eq!(damselfly_viewer.memoryspan.0, 0);
        assert_eq!(damselfly_viewer.memoryspan.1, 1024);
        damselfly_viewer.shift_memoryspan(1);
        assert_eq!(damselfly_viewer.memoryspan.0, 1024);
        assert_eq!(damselfly_viewer.memoryspan.1, 2048);
        damselfly_viewer.shift_memoryspan(2);
        assert_eq!(damselfly_viewer.memoryspan.0, 3072);
        assert_eq!(damselfly_viewer.memoryspan.1, 4096);
    }

    #[test]
    fn memory_stub_channel_test() {
        let (mut memory_stub, instruction_rx) = MemoryStub::new();
        for i in 0..5 {
            memory_stub.force_generate_event(MemoryUpdate::Allocation(i, String::from("force_generate_event_Allocation")));
        }
        for i in 0..5 {
            let incoming_instruction = instruction_rx.recv().unwrap();
            assert_eq!(incoming_instruction.get_timestamp(), i);
        }
    }

    #[test]
    fn damselfly_channel_test() {
        let (mut memory_stub, instruction_rx) = MemoryStub::new();
        let (mut damselfly, snapshot_rx) = Damselfly::new(instruction_rx);
        for i in 0..5 {
            memory_stub.force_generate_event(MemoryUpdate::Allocation(i, String::from("force_generate_event_Allocation")));
        }
        for i in 0..5 {
            damselfly.execute_instruction()
        }
        for i in 0..5 {
            let snapshot = snapshot_rx.recv().unwrap();
            assert_eq!(*snapshot.memory_map.get(&i).unwrap(), MemoryStatus::Allocated(String::from("force_generate_event_Allocation")));
        }
    }
    #[test]
    fn shift_memoryspan_right() {
        let (mut damselfly_viewer, _memory_stub) = initialise_viewer();
        damselfly_viewer.memoryspan.0 = 0;
        damselfly_viewer.memoryspan.1 = 1024;
        damselfly_viewer.shift_memoryspan(1);
        assert_eq!(damselfly_viewer.memoryspan.0, 1024);
        assert_eq!(damselfly_viewer.memoryspan.1, 2048);
        damselfly_viewer.shift_memoryspan(2);
        assert_eq!(damselfly_viewer.memoryspan.0, 3072);
        assert_eq!(damselfly_viewer.memoryspan.1, 4096);
    }

    #[test]
    fn lock_timespan() {
        let (mut damselfly_viewer, mut memory_stub) = initialise_viewer();
        for i in 0..5 {
            memory_stub.force_generate_event(MemoryUpdate::Allocation(i, String::from("force_generate_event_Allocation")));
        }
        for _ in 0..5 {
            damselfly_viewer.update();
        }
        damselfly_viewer.shift_timespan(-1);
        assert_eq!(damselfly_viewer.timespan.0, 0);
        assert_eq!(damselfly_viewer.timespan.1, 25);
        assert!(damselfly_viewer.timespan_is_unlocked);
        damselfly_viewer.lock_timespan();
        assert_eq!(damselfly_viewer.timespan.0, 0);
        assert_eq!(damselfly_viewer.timespan.1, 4);
        assert!(!damselfly_viewer.timespan_is_unlocked);
    }

    #[test]
    fn lock_memoryspan() {
        let (mut damselfly_viewer, _memory_stub) = initialise_viewer();
        damselfly_viewer.memoryspan.0 = 0;
        damselfly_viewer.memoryspan.1 = 1024;
        damselfly_viewer.shift_memoryspan(1);
        assert_eq!(damselfly_viewer.memoryspan.0, 1024);
        assert_eq!(damselfly_viewer.memoryspan.1, 2048);
        assert!(damselfly_viewer.memoryspan_is_unlocked);
        damselfly_viewer.lock_memoryspan();
        assert_eq!(damselfly_viewer.memoryspan.0, 1024);
        assert_eq!(damselfly_viewer.memoryspan.1, 2048);
    }

    #[allow(clippy::get_first)]
    #[test]
    fn stub_to_viewer_channel_test() {
        let (mut damselfly_viewer, mut memory_stub) = initialise_viewer();
        for i in 0..3 {
            memory_stub.force_generate_event(MemoryUpdate::Allocation(i, String::from("force_generate_event_Allocation")));
        }
        for i in 3..6 {
            memory_stub.force_generate_event(MemoryUpdate::PartialAllocation(i, String::from("force_generate_event_PartialAllocation")));
        }
        for i in 6..9 {
            memory_stub.force_generate_event(MemoryUpdate::Free(i - 4, String::from("force_generate_event_Free")));
        }
        for i in 0..9 {
            debug!("iteration: {i}");
            damselfly_viewer.update();
        }
        for usage in &damselfly_viewer.memory_usage_snapshots {
            assert_eq!(usage.total_memory, 65535);
        }
        assert_eq!(damselfly_viewer.memory_usage_snapshots.get(0).unwrap().memory_used_absolute, 1.0);
        assert_eq!(damselfly_viewer.memory_usage_snapshots.get(1).unwrap().memory_used_absolute, 2.0);
        assert_eq!(damselfly_viewer.memory_usage_snapshots.get(2).unwrap().memory_used_absolute, 3.0);
        assert_eq!(damselfly_viewer.memory_usage_snapshots.get(3).unwrap().memory_used_absolute, 3.5);
        assert_eq!(damselfly_viewer.memory_usage_snapshots.get(4).unwrap().memory_used_absolute, 4.0);
        assert_eq!(damselfly_viewer.memory_usage_snapshots.get(5).unwrap().memory_used_absolute, 4.5);
        assert_eq!(damselfly_viewer.memory_usage_snapshots.get(6).unwrap().memory_used_absolute, 3.5);
        assert_eq!(damselfly_viewer.memory_usage_snapshots.get(7).unwrap().memory_used_absolute, 3.0);
        assert_eq!(damselfly_viewer.memory_usage_snapshots.get(8).unwrap().memory_used_absolute, 2.5);

        let mut time = 0;
        assert_eq!(*damselfly_viewer.memory_map_snapshots.get(time).unwrap().get(&0).unwrap(), MemoryStatus::Allocated(String::from("force_generate_event_Allocation")));
        for i in 1..10 {
            assert!(!damselfly_viewer.memory_map_snapshots.get(time).unwrap().contains_key(&i));
        }

        time = 1;
        assert_eq!(*damselfly_viewer.memory_map_snapshots.get(time).unwrap().get(&0).unwrap(), MemoryStatus::Allocated(String::from("force_generate_event_Allocation")));
        assert_eq!(*damselfly_viewer.memory_map_snapshots.get(time).unwrap().get(&1).unwrap(), MemoryStatus::Allocated(String::from("force_generate_event_Allocation")));
        for i in 2..10 {
            assert!(!damselfly_viewer.memory_map_snapshots.get(time).unwrap().contains_key(&i));
        }

        time = 2;
        assert_eq!(*damselfly_viewer.memory_map_snapshots.get(time).unwrap().get(&0).unwrap(), MemoryStatus::Allocated(String::from("force_generate_event_Allocation")));
        assert_eq!(*damselfly_viewer.memory_map_snapshots.get(time).unwrap().get(&1).unwrap(), MemoryStatus::Allocated(String::from("force_generate_event_Allocation")));
        assert_eq!(*damselfly_viewer.memory_map_snapshots.get(time).unwrap().get(&2).unwrap(), MemoryStatus::Allocated(String::from("force_generate_event_Allocation")));
        for i in 3..10 {
            assert!(!damselfly_viewer.memory_map_snapshots.get(time).unwrap().contains_key(&i));
        }

        time = 3;
        assert_eq!(*damselfly_viewer.memory_map_snapshots.get(time).unwrap().get(&0).unwrap(), MemoryStatus::Allocated(String::from("force_generate_event_Allocation")));
        assert_eq!(*damselfly_viewer.memory_map_snapshots.get(time).unwrap().get(&1).unwrap(), MemoryStatus::Allocated(String::from("force_generate_event_Allocation")));
        assert_eq!(*damselfly_viewer.memory_map_snapshots.get(time).unwrap().get(&2).unwrap(), MemoryStatus::Allocated(String::from("force_generate_event_Allocation")));
        assert_eq!(*damselfly_viewer.memory_map_snapshots.get(time).unwrap().get(&3).unwrap(), MemoryStatus::PartiallyAllocated(String::from("force_generate_event_PartialAllocation")));
        for i in 4..11 {
            assert!(!damselfly_viewer.memory_map_snapshots.get(time).unwrap().contains_key(&i));
        }

        time = 4;
        assert_eq!(*damselfly_viewer.memory_map_snapshots.get(time).unwrap().get(&0).unwrap(), MemoryStatus::Allocated(String::from("force_generate_event_Allocation")));
        assert_eq!(*damselfly_viewer.memory_map_snapshots.get(time).unwrap().get(&1).unwrap(), MemoryStatus::Allocated(String::from("force_generate_event_Allocation")));
        assert_eq!(*damselfly_viewer.memory_map_snapshots.get(time).unwrap().get(&2).unwrap(), MemoryStatus::Allocated(String::from("force_generate_event_Allocation")));
        assert_eq!(*damselfly_viewer.memory_map_snapshots.get(time).unwrap().get(&3).unwrap(), MemoryStatus::PartiallyAllocated(String::from("force_generate_event_PartialAllocation")));
        assert_eq!(*damselfly_viewer.memory_map_snapshots.get(time).unwrap().get(&4).unwrap(), MemoryStatus::PartiallyAllocated(String::from("force_generate_event_PartialAllocation")));
        for i in 5..11 {
            assert!(!damselfly_viewer.memory_map_snapshots.get(time).unwrap().contains_key(&i));
        }

        time = 5;
        assert_eq!(*damselfly_viewer.memory_map_snapshots.get(time).unwrap().get(&0).unwrap(), MemoryStatus::Allocated(String::from("force_generate_event_Allocation")));
        assert_eq!(*damselfly_viewer.memory_map_snapshots.get(time).unwrap().get(&1).unwrap(), MemoryStatus::Allocated(String::from("force_generate_event_Allocation")));
        assert_eq!(*damselfly_viewer.memory_map_snapshots.get(time).unwrap().get(&2).unwrap(), MemoryStatus::Allocated(String::from("force_generate_event_Allocation")));
        assert_eq!(*damselfly_viewer.memory_map_snapshots.get(time).unwrap().get(&3).unwrap(), MemoryStatus::PartiallyAllocated(String::from("force_generate_event_PartialAllocation")));
        assert_eq!(*damselfly_viewer.memory_map_snapshots.get(time).unwrap().get(&4).unwrap(), MemoryStatus::PartiallyAllocated(String::from("force_generate_event_PartialAllocation")));
        assert_eq!(*damselfly_viewer.memory_map_snapshots.get(time).unwrap().get(&5).unwrap(), MemoryStatus::PartiallyAllocated(String::from("force_generate_event_PartialAllocation")));
        for i in 6..11 {
            assert!(!damselfly_viewer.memory_map_snapshots.get(time).unwrap().contains_key(&i));
        }

        time = 6;
        assert_eq!(*damselfly_viewer.memory_map_snapshots.get(time).unwrap().get(&0).unwrap(), MemoryStatus::Allocated(String::from("force_generate_event_Allocation")));
        assert_eq!(*damselfly_viewer.memory_map_snapshots.get(time).unwrap().get(&1).unwrap(), MemoryStatus::Allocated(String::from("force_generate_event_Allocation")));
        assert_eq!(*damselfly_viewer.memory_map_snapshots.get(time).unwrap().get(&2).unwrap(), MemoryStatus::Free(String::from("force_generate_event_Free")));
        assert_eq!(*damselfly_viewer.memory_map_snapshots.get(time).unwrap().get(&3).unwrap(), MemoryStatus::PartiallyAllocated(String::from("force_generate_event_PartialAllocation")));
        assert_eq!(*damselfly_viewer.memory_map_snapshots.get(time).unwrap().get(&4).unwrap(), MemoryStatus::PartiallyAllocated(String::from("force_generate_event_PartialAllocation")));
        assert_eq!(*damselfly_viewer.memory_map_snapshots.get(time).unwrap().get(&5).unwrap(), MemoryStatus::PartiallyAllocated(String::from("force_generate_event_PartialAllocation")));
        for i in 6..11 {
            assert!(!damselfly_viewer.memory_map_snapshots.get(time).unwrap().contains_key(&i));
        }

        time = 7;
        assert_eq!(*damselfly_viewer.memory_map_snapshots.get(time).unwrap().get(&0).unwrap(), MemoryStatus::Allocated(String::from("force_generate_event_Allocation")));
        assert_eq!(*damselfly_viewer.memory_map_snapshots.get(time).unwrap().get(&1).unwrap(), MemoryStatus::Allocated(String::from("force_generate_event_Allocation")));
        assert_eq!(*damselfly_viewer.memory_map_snapshots.get(time).unwrap().get(&2).unwrap(), MemoryStatus::Free(String::from("force_generate_event_Free")));
        assert_eq!(*damselfly_viewer.memory_map_snapshots.get(time).unwrap().get(&3).unwrap(), MemoryStatus::Free(String::from("force_generate_event_Free")));
        assert_eq!(*damselfly_viewer.memory_map_snapshots.get(time).unwrap().get(&4).unwrap(), MemoryStatus::PartiallyAllocated(String::from("force_generate_event_PartialAllocation")));
        assert_eq!(*damselfly_viewer.memory_map_snapshots.get(time).unwrap().get(&5).unwrap(), MemoryStatus::PartiallyAllocated(String::from("force_generate_event_PartialAllocation")));
        for i in 6..11 {
            assert!(!damselfly_viewer.memory_map_snapshots.get(time).unwrap().contains_key(&i));
        }

        time = 8;
        assert_eq!(*damselfly_viewer.memory_map_snapshots.get(time).unwrap().get(&0).unwrap(), MemoryStatus::Allocated(String::from("force_generate_event_Allocation")));
        assert_eq!(*damselfly_viewer.memory_map_snapshots.get(time).unwrap().get(&1).unwrap(), MemoryStatus::Allocated(String::from("force_generate_event_Allocation")));
        assert_eq!(*damselfly_viewer.memory_map_snapshots.get(time).unwrap().get(&2).unwrap(), MemoryStatus::Free(String::from("force_generate_event_Free")));
        assert_eq!(*damselfly_viewer.memory_map_snapshots.get(time).unwrap().get(&3).unwrap(), MemoryStatus::Free(String::from("force_generate_event_Free")));
        assert_eq!(*damselfly_viewer.memory_map_snapshots.get(time).unwrap().get(&4).unwrap(), MemoryStatus::Free(String::from("force_generate_event_Free")));
        assert_eq!(*damselfly_viewer.memory_map_snapshots.get(time).unwrap().get(&5).unwrap(), MemoryStatus::PartiallyAllocated(String::from("force_generate_event_PartialAllocation")));
        for i in 6..11 {
            assert!(!damselfly_viewer.memory_map_snapshots.get(time).unwrap().contains_key(&i));
        }
    }
}