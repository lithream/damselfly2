use crate::damselfly::memory::memory_update::MemoryUpdateType;

pub enum LowerPanelMode {
    SETTINGS,
    CALLSTACK,
    STATISTICS
}

pub enum MapMode {
    SNAP,
    FULL,
}

pub enum GraphMode {
    MANUAL,
    CURSOR,
}

pub struct AppDefaultState {
    pub block_size: usize,
    pub map_span: usize,
    pub map_start: usize,
    pub map_end: usize,
    pub map_mode: MapMode,
    pub graph_mode: GraphMode,
    pub current_block: Option<MemoryUpdateType>,
    pub lower_panel_mode: LowerPanelMode,
}

impl AppDefaultState {
    pub fn new(block_size: usize, map_span: usize, map_start: usize, map_end: usize, current_block: Option<MemoryUpdateType>) -> AppDefaultState {
        AppDefaultState {
            block_size,
            map_span,
            map_start,
            map_end,
            map_mode: MapMode::SNAP,
            graph_mode: GraphMode::CURSOR,
            current_block,
            lower_panel_mode: LowerPanelMode::CALLSTACK
        }
    }
}
