use std::{error};
use owo_colors::OwoColorize;
use crate::app::Mode::DEFAULT;
use crate::damselfly::consts::{DEFAULT_ROW_LENGTH};
use crate::damselfly::controller::DamselflyController;
use crate::damselfly::memory_parsers::MemorySysTraceParser;

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
    pub damselfly_controller: DamselflyController,

    pub map_grid: Vec<Vec<usize>>,
    pub graph_scale: f64,

    pub row_length: usize,
    // Actual mapspan (e.g. becomes 100 - 200 after shifting right once)
    pub up_left_width: u16,
    pub up_right_width: u16,
    pub up_middle_width: u16,
    pub up_height: u16,
    pub down_height: u16,

    pub mode: Mode
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new(trace_path: &str, binary_path: &str) -> Self {
        let mut mst_parser = MemorySysTraceParser::new();
        println!("Reading log file into memory: {}", trace_path.cyan());
        let log = std::fs::read_to_string(trace_path).unwrap();
        println!("Parsing instructions");
        let instructions = mst_parser.parse_log(log, binary_path);
        println!("Initialising DamselflyViewer");
        let mut damselfly_controller = DamselflyController::new();
        println!("Populating memory logs");
        damselfly_controller.viewer.load_instructions(instructions);
        App {
            running: true,
            damselfly_controller,
            map_grid: Vec::new(),
            graph_scale: 1.0,
            row_length: DEFAULT_ROW_LENGTH,
            up_left_width: 30,
            up_middle_width: 60,
            up_right_width: 30,
            up_height: 70,
            down_height: 30,
            mode: DEFAULT,
        }
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&mut self) {

    }

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }
}
