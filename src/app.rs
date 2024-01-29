use std::{error};
use ratatui::widgets::TableState;
use crate::app::Mode::DEFAULT;
use crate::damselfly_viewer::consts::{DEFAULT_MEMORYSPAN, DEFAULT_ROW_LENGTH};
use crate::damselfly_viewer::DamselflyViewer;
use crate::memory::MemorySysTraceParser;

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

pub enum Mode {
    DEFAULT,
    STACKTRACE,
}

/// Application.
pub struct App {
    /// Is the application running?
    pub running: bool,
    /// Damselfly
    pub damselfly_viewer: DamselflyViewer,

    pub graph_highlight: Option<usize>,
    // Always between 0 - 100
    pub map_highlight: Option<usize>,
    pub map_grid: Vec<Vec<usize>>,
    pub graph_scale: f64,

    pub row_length: usize,
    // Actual mapspan (e.g. becomes 100 - 200 after shifting right once)
    pub map_span: (usize, usize),
    pub is_mapspan_locked: bool,

    pub left_width: u16,
    pub right_width: u16,
    pub up_height: u16,
    pub down_height: u16,

    pub mode: Mode
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new(trace_path: &str, binary_path: &str, gaddr2line_path: &str) -> Self {
        let (mut mst_parser, instruction_rx) = MemorySysTraceParser::new();
        let log = std::fs::read_to_string(trace_path).unwrap();
        mst_parser.parse_log(log, binary_path, gaddr2line_path);
        let mut damselfly_viewer = DamselflyViewer::new(instruction_rx);
        damselfly_viewer.gulp_channel();
        App {
            running: true,
            damselfly_viewer,
            graph_highlight: None,
            map_highlight: None,
            map_grid: Vec::new(),
            graph_scale: 1.0,
            row_length: DEFAULT_ROW_LENGTH,
            map_span: (0, DEFAULT_MEMORYSPAN),
            is_mapspan_locked: true,
            left_width: 30,
            right_width: 70,
            up_height: 70,
            down_height: 30,
            mode: DEFAULT,
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
        let mut next_key = None;
        for key in current_map.keys() {
            if *key > current_block {
                next_key = Some(*key);
                break;
            }
        }
        /*
        let next_key = current_map.keys()
            .find(|key| **key > current_block);

         */
        if next_key.is_none() {
            return;
        }
        let next_key = next_key.unwrap();
        self.map_span.0 = next_key.saturating_sub(DEFAULT_MEMORYSPAN / 2);
        self.map_span.1 = next_key.saturating_add(DEFAULT_MEMORYSPAN / 2);
        self.map_highlight = Some(next_key);
    }

    pub fn jump_to_prev_block(&mut self) {
        if self.map_highlight.is_none() {
            return;
        }
        let current_block = self.map_highlight.unwrap();
        let (current_map, _) = self.damselfly_viewer
            .get_map_state(
                self.damselfly_viewer
                    .get_timespan().0 + self.graph_highlight
                    .unwrap_or(0));
        let mut next_key = None;
        for key in current_map.keys() {
            if *key < current_block {
                next_key = Some(*key);
            }
            if *key >= current_block {
                break;
            }
        }
        if next_key.is_none() {
            return;
        }
        let next_key = next_key.unwrap();
        self.map_span.0 = next_key.saturating_sub(DEFAULT_MEMORYSPAN / 2);
        self.map_span.1 = next_key.saturating_add(DEFAULT_MEMORYSPAN / 2);
        self.map_highlight = Some(next_key);
    }

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }
}
