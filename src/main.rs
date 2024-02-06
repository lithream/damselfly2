use map::app::{App, AppResult};
use map::event::{Event, EventHandler};
use map::handler::handle_key_events;
use map::tui::Tui;
use std::{env, fs, io};
use std::os::unix::fs::MetadataExt;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use map::damselfly::consts::{DEFAULT_BINARY_PATH, DEFAULT_LOG_PATH, DEFAULT_TICK_RATE, LARGE_FILE_TICK_RATE, MAP_CACHE_SIZE};
use owo_colors::OwoColorize;

fn main() -> AppResult<()> {
    let args: Vec<String> = env::args().collect();
    // Create an application.
    let log_path = args.get(1)
        .map(|log_path| log_path.to_string())
        .unwrap_or_else(
            || {
                eprintln!("{} No log path supplied. Using default: {DEFAULT_LOG_PATH}", String::from("[WARNING]").red());
                DEFAULT_LOG_PATH.to_string()
            });
    let binary_path = args.get(2)
        .map(|binary_path| binary_path.to_string())
        .unwrap_or_else(|| {
            eprintln!("{} No binary path supplied. Using default: {DEFAULT_BINARY_PATH}", String::from("[WARNING]").red());
            DEFAULT_BINARY_PATH.to_string()
        });
    let metadata = fs::metadata(&log_path).unwrap();
    let mut tick_rate = DEFAULT_TICK_RATE;
    if metadata.size() > 500000000 {
        let info = String::from("[INFO]");
        let info = info.yellow();
        println!("{} Log size: {} bytes. Large logs may degrade performance.", info, metadata.size().red());
        println!("{} Caching snapshots of the log every {} operations lowers latency at the cost of higher RAM usage.", info, MAP_CACHE_SIZE.cyan());
        println!("{} Slowing the TUI tick rate from {}ms to {}ms prevents CPU throttling at the cost of a slight delay when refreshing the memory map window.", info, DEFAULT_TICK_RATE.cyan(), LARGE_FILE_TICK_RATE.cyan());
        println!("{} {} to enable these optimisations. {} to ignore.", info, String::from("y/Y").green(), String::from("n/N").red());
        let mut input = String::new();
        loop {
            input.clear();
            io::stdin().read_line(&mut input).unwrap();
            let input = input.trim();
            match input {
                "y" => {
                    tick_rate = LARGE_FILE_TICK_RATE;
                    break;
                }
                "n" => {
                    tick_rate = DEFAULT_TICK_RATE;
                    break;
                }
                _ => {
                    println!("Invalid response. Please enter y/Y or n/N.");
                }
            }
        }
    }

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
