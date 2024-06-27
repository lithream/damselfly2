//! Memory map component.
//!
//! Most of these methods are called in DamselflyInstance. Consult its documentation to see
//! how they might be used.
use std::cmp::{max, min};

use crate::damselfly::consts::{DEFAULT_BLOCK_SIZE, DEFAULT_MEMORYSPAN};
use crate::damselfly::memory::memory_cache::MemoryCache;
use crate::damselfly::memory::memory_status::MemoryStatus;
use crate::damselfly::memory::memory_update::MemoryUpdateType;
use crate::damselfly::update_interval::UpdateInterval;

pub struct MapViewer {
    map_name: String,
    cache: MemoryCache,
    update_intervals: Vec<UpdateInterval>,
    current_timestamp: usize,
    canvas_start: usize,
    canvas_span: usize,
    block_size: usize,
    lowest_address: usize,
    highest_address: usize,
}

impl MapViewer {
    pub fn new(map_name: String, update_intervals: Vec<UpdateInterval>, lowest_address: usize, highest_address: usize, cache_size: u64) -> MapViewer {
        let current_timestamp = update_intervals.len().saturating_sub(1);

        let analysed_lowest_address = update_intervals.iter().min_by(|prev, next| {
            prev.val.get_absolute_address().cmp(&next.val.get_absolute_address())
        }).expect("[MapViewer::new]: Cannot find lowest address").val.get_absolute_address();

        let analysed_highest_address = update_intervals.iter().max_by(|prev, next| {
            prev.val.get_absolute_address().cmp(&next.val.get_absolute_address())
        }).expect("[MapViewer::new]: Cannot find highest address").val.get_absolute_address();

        println!("Reported pool bounds from log: {lowest_address} -> {highest_address}");
        println!("Analysed pool bounds from instructions: {analysed_lowest_address} -> {analysed_highest_address}");
        println!("The reported pool bounds should be larger than or equal to the analysed bounds.");

        MapViewer {
            map_name,
            cache: MemoryCache::new(DEFAULT_BLOCK_SIZE, update_intervals.clone(), cache_size as usize),
            update_intervals,
            current_timestamp,
            canvas_start: 0,
            canvas_span: DEFAULT_MEMORYSPAN,
            block_size: DEFAULT_BLOCK_SIZE,
            lowest_address: min(lowest_address, analysed_lowest_address),
            highest_address: max(highest_address, analysed_highest_address),
        }
    }

    pub fn get_update_history(&self, history_size: usize) -> Vec<MemoryUpdateType> {
        println!("[get_update_history]: current timestamp: {}", self.current_timestamp);
        let mut update_history = Vec::new();
        for update in &self.update_intervals {
            if update.val.get_timestamp() > self.current_timestamp {
                break;
            }
            update_history.push(update);
        }
        update_history
            .iter()
            .rev()
            .take(history_size)
            .map(|update_interval| update_interval.val.clone())
            .collect()
    }
    
    pub fn set_timestamp(&mut self, new_timestamp: usize) {
        self.current_timestamp = new_timestamp.clamp(usize::MIN, self.update_intervals.last().unwrap().val.get_timestamp());
    }
    
    pub fn set_map_span(&mut self, new_span: usize) {
        self.canvas_span = new_span;
    }

    pub fn get_block_size(&self) -> usize {
        self.block_size
    }
    
    pub fn set_block_size(&mut self, new_size: usize) {
        let span_scale_factor = new_size as f64 / self.block_size as f64;
        self.set_map_span((self.canvas_span as f64 * span_scale_factor).round() as usize);
        self.block_size = new_size;
        self.cache.change_block_size(new_size);
    }

    pub fn paint_map_full_from_cache(&self) -> Vec<MemoryStatus> {
        self.cache.query_cache(self.current_timestamp).unwrap()
    }

    pub fn get_current_operation(&self) -> MemoryUpdateType {
        match self.update_intervals.get(self.current_timestamp) {
            None => {
                for timestamp in (0..self.current_timestamp).rev() {
                    match self.update_intervals.get(timestamp) {
                        None => continue,
                        Some(update) => return update.val.clone()
                    }
                }
                panic!("[MapViewer::get_current_operation]: No operation found at timestamp: {}", self.current_timestamp);
            }
            Some(update) => update.val.clone()
        }
    }
}