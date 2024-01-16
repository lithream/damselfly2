use std::{error, thread};
use crate::damselfly::damselfly_viewer::DamselflyViewer;
use crate::damselfly::Damselfly;
use crate::memory::MemoryStub;

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

/// Application.
pub struct App {
    /// Is the application running?
    pub running: bool,
    /// Damselfly
    pub damselfly_viewer: DamselflyViewer,
    pub highlight: Option<usize>,
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Self {
        let (mut memory_stub, instruction_rx) = MemoryStub::new();
        thread::spawn(move ||{
            loop {
                memory_stub.generate_event();
            }
        });
        let (damselfly, snapshot_rx) = Damselfly::new(instruction_rx);
        let damselfly_viewer = DamselflyViewer::new(damselfly, snapshot_rx);
        App {
            running: true,
            damselfly_viewer,
            highlight: None,
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
