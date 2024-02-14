use rust_lapper::Lapper;
use crate::damselfly2::memory::memory_update::MemoryUpdateType;
use crate::damselfly2::update_interval::update_interval_sorter::UpdateIntervalSorter;
use crate::damselfly2::update_interval::UpdateInterval;
use crate::damselfly2::consts;
use crate::damselfly2::memory::memory_status::MemoryStatus;
use crate::damselfly2::memory::NoHashMap;

pub struct MapViewer {
    lapper: Lapper<usize, MemoryUpdateType>,
    current_highlight: usize,
    start: usize,
    stop: usize,
    block_size: usize,
}

impl MapViewer {
    pub fn new(update_intervals: Vec<UpdateInterval>) -> MapViewer {
        MapViewer {
            lapper: Lapper::new(update_intervals),
            current_highlight: 0,
            start: 0,
            stop: consts::DEFAULT_MEMORYSPAN,
            block_size: consts::DEFAULT_BLOCK_SIZE,
        }
    }

    pub fn get_operations_in_window(&self, start: usize, end: usize) -> Vec<&UpdateInterval> {
        let mut intervals = self.lapper.find(start, end).collect::<Vec<&UpdateInterval>>();
        UpdateIntervalSorter::sort_by_timestamp(&mut intervals);
        intervals
    }

    pub fn pan(&mut self, units: usize) {
        self.start -= units;
        self.stop -= units;
    }

    pub fn paint_map(&self) -> NoHashMap<usize, MemoryStatus> {
        NoHashMap::default()
    }
}