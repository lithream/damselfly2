use map::app::{App, AppResult};
use map::event::{Event, EventHandler};
use map::handler::handle_key_events;
use map::tui::Tui;
use std::{env, io};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use map::damselfly_viewer::consts::{DEFAULT_BINARY_PATH, DEFAULT_GADDR2LINE_PATH, DEFAULT_LOG_PATH};

fn main() -> AppResult<()> {
    let args: Vec<String> = env::args().collect();
    // Create an application.
    let log_path = args.get(1)
        .map(|log_path| log_path.to_string())
        .unwrap_or(DEFAULT_LOG_PATH.to_string());
    let binary_path = args.get(2)
        .map(|binary_path| binary_path.to_string())
        .unwrap_or(DEFAULT_BINARY_PATH.to_string());
    let gaddr2line_path = args.get(3)
        .map(|gaddr2line_path| gaddr2line_path.to_string())
        .unwrap_or(DEFAULT_GADDR2LINE_PATH.to_string());
    let mut app = App::new(log_path.as_str(), binary_path.as_str(), gaddr2line_path.as_str());

    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(50);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    // Start the main loop.
    while app.running {
        // Render the user interface.
        tui.draw(&mut app)?;
        // Handle events.
        match tui.events.next()? {
            Event::Tick => app.tick(),
            Event::Key(key_event) => handle_key_events(key_event, &mut app)?,
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
        }
    }

    // Exit the user interface.
    tui.exit()?;
    Ok(())
}
