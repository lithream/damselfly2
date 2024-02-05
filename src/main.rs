use map::app::{App, AppResult};
use map::event::{Event, EventHandler};
use map::handler::handle_key_events;
use map::tui::Tui;
use std::{env, fs, io};
use std::fs::File;
use std::os::unix::fs::MetadataExt;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use map::damselfly_viewer::consts::{DEFAULT_BINARY_PATH, DEFAULT_LOG_PATH, DEFAULT_TICK_RATE, LARGE_FILE_TICK_RATE};

fn main() -> AppResult<()> {
    let args: Vec<String> = env::args().collect();
    // Create an application.
    let log_path = args.get(1)
        .map(|log_path| log_path.to_string())
        .unwrap_or_else(
            || {
                eprintln!("No log path supplied. Using default: {DEFAULT_LOG_PATH}");
                DEFAULT_LOG_PATH.to_string()
            });
    let binary_path = args.get(2)
        .map(|binary_path| binary_path.to_string())
        .unwrap_or_else(|| {
            eprintln!("No binary path supplied. Using default: {DEFAULT_BINARY_PATH}");
            DEFAULT_BINARY_PATH.to_string()
        });
    let metadata = fs::metadata(&log_path).unwrap();
    let tick_rate: u64 = if metadata.size() > 500000000 {
        LARGE_FILE_TICK_RATE
    } else {
        DEFAULT_TICK_RATE
    };

    let mut app = App::new(log_path.as_str(), binary_path.as_str());
    app.graph_highlight = Some(0);

    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(tick_rate);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    // Start the main loop.
    while app.running {
        // Render the user interface.
        tui.draw(&mut app)?;
        // Handle events.
        match tui.events.next()? {
            Event::Tick => {}
            Event::Key(key_event) => handle_key_events(key_event, &mut app)?,
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
        }
    }

    // Exit the user interface.
    tui.exit()?;
    Ok(())
}
