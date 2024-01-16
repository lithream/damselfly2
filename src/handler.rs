use std::ops::Add;
use crate::app::{App, AppResult};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handles the key events and updates the state of [`App`].
pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match key_event.code {
        // Exit application on `ESC` or `q`
        KeyCode::Esc | KeyCode::Char('q') => {
            app.quit();
        }
        // Exit application on `Ctrl-C`
        KeyCode::Char('c') | KeyCode::Char('C') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                app.quit();
            }
        }

        KeyCode::Char('/') => {
            app.damselfly_viewer.unlock_timespan()
        }

        KeyCode::Char('?') => {
            app.highlight = None;
            app.damselfly_viewer.lock_timespan();
        }

        KeyCode::Char('H') => {
            app.damselfly_viewer.shift_timespan(-1);
        }

        KeyCode::Char('L') => {
            app.damselfly_viewer.shift_timespan(1);
        }

        KeyCode::Char('h') => {
            match app.highlight {
                None => {
                    let span = app.damselfly_viewer.get_span();
                    app.highlight = Some((span.1 - span.0) / 2);
                }
                Some(highlight) => {
                    let span = app.damselfly_viewer.get_span();
                    app.highlight = Some(highlight.saturating_sub(1));
                }
            }
        }

        KeyCode::Char('l') => {
            match app.highlight {
                None => {
                    let span = app.damselfly_viewer.get_span();
                    app.highlight = Some((span.1 - span.0) / 2);
                }
                Some(highlight) => {
                    let span = app.damselfly_viewer.get_span();
                    app.highlight = Some((highlight + 1).clamp(0, span.1 - span.0 - 1));
                }
            }
        }

        KeyCode::Char('0') => {
            app.highlight = Some(0);
        }

        KeyCode::Char('$') => {
            let span = app.damselfly_viewer.get_span();
            app.highlight = Some(span.1 - span.0 - 1);
        }

        // Counter handlers
        KeyCode::Right => {
        }
        KeyCode::Left => {
        }
        // Other handlers you could add here.
        _ => {}
    }
    Ok(())
}
