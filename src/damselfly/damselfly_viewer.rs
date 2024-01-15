use crate::damselfly::Damselfly;


const DEFAULT_TIMESPAN: u64 = 25;
const DEFAULT_MEMORYSPAN: u64 = 1024;
struct DamselflyViewer {
    damselfly: Damselfly,
    timespan: (u64, u64),
    timespan_is_unlocked: bool,
    memoryspan: (u64, u64),
    memoryspan_is_unlocked: bool
}

impl DamselflyViewer {
    pub fn new(damselfly: Damselfly) -> DamselflyViewer {
        DamselflyViewer{
            damselfly,
            timespan: (0, DEFAULT_TIMESPAN),
            timespan_is_unlocked: false,
            memoryspan: (0, DEFAULT_MEMORYSPAN),
            memoryspan_is_unlocked: false,
        }
    }

    fn shift_span(&mut self, mut left: u64, mut right: u64, units: i64) -> (u64, u64) {
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
        match self.damselfly.get_latest_instruction() {
            None => self.timespan.1 = 0,
            Some(instruction) => self.timespan.1 = instruction.get_timestamp() as u64 + 1
        }
        self.timespan.0 = self.timespan.1 - current_span;
        self.timespan_is_unlocked = false;
    }

    pub fn shift_memoryspan(&mut self, units: i64) {
        self.memoryspan_is_unlocked = true;
        (self.memoryspan.0, self.memoryspan.1) = self.shift_span(self.memoryspan.0, self.memoryspan.1, units);
    }

    pub fn lock_memoryspan(&mut self) {
        self.memoryspan_is_unlocked = false;
    }
    
    pub fn update_view(&mut self) {
        
    }
}

#[cfg(test)]
mod tests {
    use crate::damselfly::Damselfly;
    use crate::damselfly::damselfly_viewer::DamselflyViewer;
    use crate::damselfly::instruction::Instruction;
    use crate::memory::MemoryStub;
    use crate::memory::MemoryUpdate::Allocation;

    fn initialise_viewer() -> (DamselflyViewer, MemoryStub) {
        let (memory_stub, rx) = MemoryStub::new();
        let damselfly = Damselfly::new(rx);
        let damselfly_viewer = DamselflyViewer::new(damselfly);
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
        let (mut damselfly_viewer, _memory_stub) = initialise_viewer();
        damselfly_viewer.timespan.0 = 25;
        damselfly_viewer.timespan.1 = 50;
        for i in 0..50 {
            damselfly_viewer.damselfly.instruction_history.push(Instruction::new(i, Allocation(i)));
        }
        damselfly_viewer.shift_timespan(-1);
        assert_eq!(damselfly_viewer.timespan.0, 0);
        assert_eq!(damselfly_viewer.timespan.1, 25);
        assert!(damselfly_viewer.timespan_is_unlocked);
        damselfly_viewer.lock_timespan();
        assert_eq!(damselfly_viewer.timespan.0, 25);
        assert_eq!(damselfly_viewer.timespan.1, 50);
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
}