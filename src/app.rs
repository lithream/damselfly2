use std::{error};
use ratatui::widgets::TableState;
use crate::damselfly_viewer::consts::{DEFAULT_MEMORYSPAN, DEFAULT_ROW_LENGTH};
use crate::damselfly_viewer::DamselflyViewer;
use crate::memory::MemorySysTraceParser;

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

/// Application.
pub struct App {
    /// Is the application running?
    pub running: bool,
    /// Damselfly
    pub damselfly_viewer: DamselflyViewer,

    pub graph_highlight: Option<usize>,
    pub map_highlight: Option<usize>,
    pub map_grid: Vec<Vec<usize>>,
    pub graph_scale: f64,
    pub table_state: TableState,

    pub row_length: usize,
    pub map_span: (usize, usize),
    pub is_mapspan_locked: bool,
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Self {
        let (mut mst_parser, instruction_rx) = MemorySysTraceParser::new();
        let log = std::fs::read_to_string("trace.log").unwrap();
        mst_parser.parse_log(log);
        let mut damselfly_viewer = DamselflyViewer::new(instruction_rx);
        damselfly_viewer.gulp_channel();
        App {
            running: true,
            damselfly_viewer,
            graph_highlight: None,
            map_highlight: None,
            map_grid: Vec::new(),
            graph_scale: 1.0,
            table_state: TableState::default(),
            row_length: DEFAULT_ROW_LENGTH,
            map_span: (0, DEFAULT_MEMORYSPAN),
            is_mapspan_locked: true,
        }
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&mut self) {

    }

    pub fn jump_to_next_block(&mut self) {
        if self.map_highlight.is_none() {
            return;
        }
        let current_block = self.map_highlight.unwrap();
        let (current_map, _) = self.damselfly_viewer
            .get_map_state(
                self.damselfly_viewer
                    .get_timespan().0 + self.graph_highlight
                    .unwrap_or(0));
        let next_key = current_map.keys()
            .find(|key| **key > current_block);
        if next_key.is_none() {
            return;
        }
        let next_key = next_key.unwrap();
        self.map_span.0 = next_key.saturating_sub(DEFAULT_MEMORYSPAN);
        self.map_span.1 = next_key.saturating_add(DEFAULT_MEMORYSPAN);
        self.map_highlight = Some(*next_key);
    }

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }
}
