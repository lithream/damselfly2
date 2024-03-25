pub struct MemorySnapshot {
    lapper: Lapper<UpdateInterval>
}

impl MemorySnapshot {
    pub fn new(update_intervals: Vec<UpdateInterval>) -> Self {
        Self {
            lapper: Lapper::new(update_intervals)
        }
    }
}