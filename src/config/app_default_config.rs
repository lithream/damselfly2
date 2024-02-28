use crate::damselfly::consts::DEFAULT_BLOCKS_BEFORE_TRUNCATE;
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

pub enum GraphTimeMode {
    OPERATION,
    REALTIME,
}

pub struct AppDefaultState {
    pub block_size: usize,
    pub pixel_size: f32,
    pub map_span: usize,
    pub map_start: usize,
    pub map_end: usize,
    pub blocks_before_truncate: usize,
    pub map_mode: MapMode,
    pub graph_time_mode: GraphTimeMode,
    pub graph_mode: GraphMode,
    pub current_block: Option<MemoryUpdateType>,
    pub lower_panel_mode: LowerPanelMode,
}

impl AppDefaultState {
    pub fn new(block_size: usize, pixel_size: f32, map_span: usize, map_start: usize, map_end: usize, blocks_before_truncate: usize, current_block: Option<MemoryUpdateType>) -> AppDefaultState {
        AppDefaultState {
            block_size,
            pixel_size,
            map_span,
            map_start,
            map_end,
            blocks_before_truncate,
            map_mode: MapMode::SNAP,
            graph_mode: GraphMode::CURSOR,
            graph_time_mode: GraphTimeMode::OPERATION,
            current_block,
            lower_panel_mode: LowerPanelMode::CALLSTACK
        }
    }
}
