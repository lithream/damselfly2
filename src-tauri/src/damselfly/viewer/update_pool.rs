use crate::damselfly::update_interval::UpdateInterval;

pub struct UpdatePool {
    start: u64,
    end: u64,
    update_intervals: Vec<UpdateInterval>,
}

impl UpdatePool {
    pub fn new(start: u64, end: u64, update_intervals: Vec<UpdateInterval>) -> Self {
        Self {
            start,
            end,
            update_intervals
        }
    }
}