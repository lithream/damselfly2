use crate::damselfly::memory::memory_update::MemoryUpdateType;

pub struct AppDefaultConfig {
    pub(crate) block_size: usize,
    pub(crate) map_span: usize,
    pub(crate) current_block: Option<MemoryUpdateType>,
}

impl AppDefaultConfig {
    pub fn new(block_size: usize, map_span: usize, current_block: Option<MemoryUpdateType>) -> AppDefaultConfig {
        AppDefaultConfig {
            block_size,
            map_span,
            current_block,
        }
    }
}
