use crate::damselfly::memory::memory_update::MemoryUpdateType;

pub struct AppDefaultState {
    pub block_size: usize,
    pub map_span: usize,
    pub current_block: Option<MemoryUpdateType>,
}

impl AppDefaultState {
    pub fn new(block_size: usize, map_span: usize, current_block: Option<MemoryUpdateType>) -> AppDefaultState {
        AppDefaultState {
            block_size,
            map_span,
            current_block,
        }
    }
}
