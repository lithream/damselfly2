use crate::damselfly::memory::memory_status::MemoryStatus;
use crate::damselfly::memory::memory_update::MemoryUpdateType;

pub struct AppMemoryMapState {
    block_size: usize,
    map_span: usize,
    current_block: Option<MemoryUpdateType>,
    // (timestamp, Vec<rect, color>)
    cached_map: Option<(usize, Vec<(egui::Rect, egui::Color32)>)>,
}

impl AppMemoryMapState {
    pub fn new(block_size: usize, map_span: usize, current_block: Option<MemoryUpdateType>) -> AppMemoryMapState {
        AppMemoryMapState {
            block_size,
            map_span,
            current_block,
            cached_map: None,
        }
    }
    
    pub fn cache_map(&mut self, map: (usize, Vec<(egui::Rect, egui::Color32)>)) {
        self.cached_map = Some(map);
    }
    
    pub fn get_cached_map(&self) -> &Option<(usize, Vec<(egui::Rect, egui::Color32)>)> {
        &self.cached_map
    }
}