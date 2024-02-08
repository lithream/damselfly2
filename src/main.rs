/// Application.
pub mod app;
/// Widget renderer.
pub mod ui;
/// Damselfly
pub mod damselfly;
mod consts;

use std::{env, fs, io};
use iced::{Application, Settings};
use map::damselfly::consts::{DEFAULT_BINARY_PATH, DEFAULT_LOG_PATH};
use owo_colors::OwoColorize;
use crate::app::{App, Flags};

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> iced::Result {
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

    let flags: Flags = Flags {
        log_path,
        binary_path
    };

    App::run(Settings::with_flags(flags))
}