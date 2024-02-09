use crate::damselfly::memory_parsers::MemorySysTraceParser;
use std::path::Path;
use std::sync::Arc;
use iced::{Command, Element, executor, Renderer, Application, Theme};
use owo_colors::OwoColorize;
use tokio::io;
use crate::damselfly::controller::DamselflyController;

pub struct App {
    pub damselfly_controller: DamselflyController,
    pub graph_highlight: usize,
}

#[derive(Debug, Clone)]
pub enum Message {
    DamselflyReady,
}

#[derive(Default)]
pub struct Flags {
    pub log_path: String,
    pub binary_path: String,
}

impl Application for App {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = Flags;

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            Self {
                damselfly_controller: DamselflyController::new(),
                graph_highlight: 0
            },
            Command::perform(
                initialise_damselfly(flags.log_path, flags.binary_path),
                |_| Message::DamselflyReady
            ),
        )
    }

    fn title(&self) -> String {
        String::from("DAMSELFLY")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        Command::none()
    }

    fn view(&self) -> Element<'_, Self::Message, Renderer<Self::Theme>> {
        todo!()
    }

    fn theme(&self) -> Self::Theme {
        Theme::Dark
    }
}

async fn load_file(path: impl AsRef<Path>) -> Result<Arc<String>, io::ErrorKind> {
    tokio::fs::read_to_string(path).await.map(Arc::new).map_err(|error| error.kind())
}

async fn initialise_damselfly(log_path: String, binary_path: String) -> Result<(), Error> {
    let mut mst_parser = MemorySysTraceParser::new();
    println!("Reading log file into memory: {}", log_path.cyan());
    // todo: add native file picker
    let log = std::fs::read_to_string(log_path).unwrap();
    println!("Parsing instructions");
    let instructions = mst_parser.parse_log(log, binary_path);
    println!("Initialising DamselflyViewer");
    let mut damselfly_controller = DamselflyController::new();
    println!("Populating memory logs");
    damselfly_controller.viewer.load_instructions(instructions);
    Ok(())
}

#[derive(Debug, Clone)]
pub enum Error {
    DialogClosed,
    IO(io::ErrorKind),
}