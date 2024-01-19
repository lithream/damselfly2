use crate::app::{App, AppResult};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::damselfly_viewer::consts::{DEFAULT_MEMORY_SIZE, DEFAULT_ROW_LENGTH};

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

        KeyCode::Char(';') => {
            app.damselfly_viewer.unlock_timespan()
        }

        KeyCode::Char(':') => {
            app.damselfly_viewer.lock_timespan();
        }

        KeyCode::Char('H') => {
            app.damselfly_viewer.shift_timespan_left(1);
        }

        KeyCode::Char('L') => {
            app.damselfly_viewer.shift_timespan_right(1);
        }

        KeyCode::Char('h') => {
            match app.graph_highlight {
                None => {
                    let span = app.damselfly_viewer.get_timespan();
                    app.graph_highlight = Some((span.1 - span.0) / 2);
                }
                Some(highlight) => {
                    app.graph_highlight = Some(highlight.saturating_sub(1));
                }
            }
        }

        KeyCode::Char('l') => {
            match app.graph_highlight {
                None => {
                    let span = app.damselfly_viewer.get_timespan();
                    app.graph_highlight = Some((span.1 - span.0) / 2);
                }
                Some(highlight) => {
                    let span = app.damselfly_viewer.get_timespan();
                    app.graph_highlight = Some((highlight + 1).clamp(0, span.1 - span.0 - 1));
                }
            }
        }

        KeyCode::Char('i') => {
            app.damselfly_viewer.unlock_timespan();
            app.damselfly_viewer.shift_timespan_to_beginning();
        }

        KeyCode::Char('o') => {
            app.damselfly_viewer.lock_timespan();
            app.damselfly_viewer.unlock_timespan();
            app.damselfly_viewer.shift_timespan_to_end();
        }

        KeyCode::Char('0') => {
            app.graph_highlight = Some(0);
        }

        KeyCode::Char('$') => {
            let span = app.damselfly_viewer.get_timespan();
            app.graph_highlight = Some(span.1 - span.0 - 1);
        }

        KeyCode::Char('=') => {
            app.graph_scale *= 2.0;
        }

        KeyCode::Char('-') => {
            app.graph_scale /= 2.0;
        }

        KeyCode::Char('j') => {
            app.map_highlight = Some(app.map_highlight.unwrap_or(0).saturating_sub(1));
        }

        KeyCode::Char('J') => {
            app.map_highlight = Some(app.map_highlight.unwrap_or(0).saturating_add(DEFAULT_ROW_LENGTH));
        }

        KeyCode::PageUp => {
            app.map_span.0 = app.map_span.0.saturating_sub(64);
            app.map_span.1 = app.map_span.1.saturating_sub(64);
            app.map_highlight = Some(app.map_highlight.unwrap_or(0).saturating_sub(DEFAULT_ROW_LENGTH));
        }

        KeyCode::PageDown => {
            app.map_span.0 = app.map_span.0.saturating_add(64);
            app.map_span.1 = app.map_span.1.saturating_add(64);
            app.map_highlight = Some(app.map_highlight.unwrap_or(0).saturating_add(DEFAULT_ROW_LENGTH));
        }

        KeyCode::Left => {
            app.map_highlight = Some(app.map_highlight.unwrap_or(0).saturating_sub(1));
        }

        KeyCode::Right => {
            app.map_highlight = Some(app.map_highlight.unwrap_or(0).saturating_add(1));
        }

        KeyCode::Up => {
            app.map_highlight = Some(app.map_highlight.unwrap_or(0).saturating_sub(DEFAULT_ROW_LENGTH));
        }

        KeyCode::Down => {
            app.map_highlight = Some(app.map_highlight.unwrap_or(0).saturating_add(DEFAULT_ROW_LENGTH));
        }

        KeyCode::Char('k') => {
            app.map_highlight = Some(app.map_highlight.unwrap_or(0).saturating_add(1));
        }

        KeyCode::Char('K') => {
            app.map_highlight = Some(app.map_highlight.unwrap_or(0).saturating_sub(DEFAULT_ROW_LENGTH));
        }

        // Other handlers you could add here.
        _ => {}
    }
    Ok(())
}
