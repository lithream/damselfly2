use std::{error, thread};
use ratatui::widgets::TableState;
use crate::damselfly_viewer::consts::{DEFAULT_MEMORYSPAN, DEFAULT_ROW_LENGTH};
use crate::damselfly_viewer::DamselflyViewer;
use crate::memory::MemoryStub;

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
        let (mut memory_stub, instruction_rx) = MemoryStub::new();
        thread::spawn(move ||{
            loop {
                memory_stub.generate_event_sequential();
            }
        });
        let damselfly_viewer = DamselflyViewer::new(instruction_rx);
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
        self.damselfly_viewer.update();
    }

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }
}
