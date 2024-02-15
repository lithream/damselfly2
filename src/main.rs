use std::{env, fs, io};
use map::damselfly::consts::{DEFAULT_BINARY_PATH, DEFAULT_LOG_PATH};
use owo_colors::OwoColorize;
use map::app::App;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    
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

    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 220.0])
            .with_icon(
                // NOE: Adding an icon is optional
                eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon-256.png")[..])
                    .unwrap(),
            ),
        ..Default::default()
    };
    eframe::run_native(
        "eframe template",
        native_options,
        Box::new(|cc| Box::new(App::new(cc, log_path, binary_path))),
    )
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "the_canvas_id", // hardcode it
                web_options,
                Box::new(|cc| Box::new(App::new(cc, log_path, binary_path))),
            )
            .await
            .expect("failed to start eframe");
    });
}