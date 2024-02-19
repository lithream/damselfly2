use crate::damselfly::memory::memory_update::MemoryUpdateType;

pub struct AppMemoryMapConfig {
    block_size: usize,
    map_span: usize,
    current_block: Option<MemoryUpdateType>,
}

impl AppMemoryMapConfig {
    pub fn new(block_size: usize, map_span: usize, current_block: Option<MemoryUpdateType>) -> AppMemoryMapConfig {
        AppMemoryMapConfig {
            block_size,
            map_span,
            current_block,
        }
    }
}