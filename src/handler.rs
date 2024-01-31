use std::cmp::{max, min};
use crate::app::{App, AppResult};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crate::app::Mode::{DEFAULT, STACKTRACE};
use crate::damselfly_viewer::consts::{DEFAULT_BLOCK_SIZE, DEFAULT_MEMORYSPAN, DEFAULT_TIMESPAN, MIN_ROW_LENGTH};

/// Handles the key events and updates the state of [`App`].
pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match key_event.code {
        // Exit application on `ESC` or `q`
        KeyCode::Esc | KeyCode::Char('q') => {
            app.quit();
        }
        // Exit application on `Ctrl-C`
        KeyCode::Char('c') | KeyCode::Char('C') => {
            if key_event.kind == KeyEventKind::Press && key_event.modifiers == KeyModifiers::CONTROL {
                app.quit();
            }
        }

        KeyCode::Char(';') => {
            if key_event.kind == KeyEventKind::Press {
                app.damselfly_viewer.unlock_timespan()
            }
        }

        KeyCode::Char(':') => {
            if key_event.kind == KeyEventKind::Press {
                app.damselfly_viewer.lock_timespan();
            }
        }

        KeyCode::Char('\'') => {
            if key_event.kind == KeyEventKind::Press {
                app.is_mapspan_locked = !app.is_mapspan_locked;
            }
        }

        KeyCode::Char('H') => {
            if key_event.kind == KeyEventKind::Press {
                app.damselfly_viewer.shift_timespan_left(1);
                app.graph_highlight = Some(DEFAULT_TIMESPAN);
            }
        }

        KeyCode::Char('L') => {
            if key_event.kind == KeyEventKind::Press {
                app.damselfly_viewer.shift_timespan_right(1);
                app.graph_highlight = Some(0);
            }
        }

        KeyCode::Char('h') => {
            if key_event.kind == KeyEventKind::Press {
                match app.graph_highlight {
                    None => {
                        let span = app.damselfly_viewer.get_timespan();
                        app.graph_highlight = Some((span.1 - span.0) / 2);
                    }
                    Some(highlight) => {
                        let span = app.damselfly_viewer.get_timespan();
                        if app.graph_highlight.unwrap() + span.0 == span.0 {
                            app.damselfly_viewer.shift_timespan_left(1);
                            app.graph_highlight = Some(DEFAULT_TIMESPAN);
                        } else {
                            app.graph_highlight = Some(highlight.saturating_sub(1));
                        }
                    }
                }
            }
        }

        KeyCode::Char('l') => {
            if key_event.kind == KeyEventKind::Press {
                match app.graph_highlight {
                    None => {
                        let span = app.damselfly_viewer.get_timespan();
                        app.graph_highlight = Some((span.1 - span.0) / 2);
                    }
                    Some(highlight) => {
                        let span = app.damselfly_viewer.get_timespan();
                        if app.graph_highlight.unwrap() + span.0 == span.1 - 1 {
                            app.damselfly_viewer.shift_timespan_right(1);
                            app.graph_highlight = Some(0);
                        } else {
                            app.graph_highlight = Some((highlight + 1).clamp(0, span.1 - span.0 - 1));
                        }
                    }
                }
            }
        }

        KeyCode::Char('i') => {
            if key_event.kind == KeyEventKind::Press {
                app.graph_highlight = Some(0);
            }
        }

        KeyCode::Char('o') => {
            if key_event.kind == KeyEventKind::Press {
                let span = app.damselfly_viewer.get_timespan();
                app.graph_highlight = Some(span.1 - span.0 - 1);
            }
        }

        KeyCode::Char('[') => {
            if key_event.kind == KeyEventKind::Press {
                app.damselfly_viewer.unlock_timespan();
                app.damselfly_viewer.shift_timespan_to_beginning();
            }
        }

        KeyCode::Char(']') => {
            if key_event.kind == KeyEventKind::Press {
                app.damselfly_viewer.lock_timespan();
                app.damselfly_viewer.unlock_timespan();
                app.damselfly_viewer.shift_timespan_to_end();
            }
        }

        KeyCode::Char('=') => {
            if key_event.kind == KeyEventKind::Press {
                app.graph_scale *= 2.0;
            }
        }

        KeyCode::Char('-') => {
            if key_event.kind == KeyEventKind::Press {
                app.graph_scale /= 2.0;
            }
        }

        KeyCode::Char('j') => {
            if key_event.kind == KeyEventKind::Press {
                app.map_highlight = Some(app.map_highlight.unwrap_or(0).saturating_sub(1));
            }
        }

        KeyCode::Char('J') => {
            if key_event.kind == KeyEventKind::Press {
                app.map_highlight = Some(app.map_highlight.unwrap_or(0).saturating_add(app.row_length));
            }
        }

        KeyCode::PageUp => {
            if key_event.kind == KeyEventKind::Press {
                if key_event.modifiers == KeyModifiers::SHIFT {
                    app.map_span.0 = app.map_span.0.saturating_sub(DEFAULT_MEMORYSPAN);
                    app.map_span.1 = app.map_span.1.saturating_sub(DEFAULT_MEMORYSPAN);
                    app.map_highlight = Some(app.map_highlight.unwrap_or(0).saturating_sub(DEFAULT_MEMORYSPAN));
                } else {
                    app.map_span.0 = app.map_span.0.saturating_sub(app.row_length);
                    app.map_span.1 = app.map_span.1.saturating_sub(app.row_length);
                    app.map_highlight = Some(app.map_highlight.unwrap_or(0).saturating_sub(app.row_length));
                }
            }
        }

        KeyCode::PageDown => {
            if key_event.kind == KeyEventKind::Press {
                if key_event.modifiers == KeyModifiers::SHIFT {
                    app.map_span.0 = app.map_span.0.saturating_add(DEFAULT_MEMORYSPAN);
                    app.map_span.1 = app.map_span.1.saturating_add(DEFAULT_MEMORYSPAN);
                    app.map_highlight = Some(app.map_highlight.unwrap_or(0).saturating_add(DEFAULT_MEMORYSPAN));
                }
                app.map_span.0 = app.map_span.0.saturating_add(app.row_length);
                app.map_span.1 = app.map_span.1.saturating_add(app.row_length);
                app.map_highlight = Some(app.map_highlight.unwrap_or(0).saturating_add(app.row_length));
            }
        }

        KeyCode::Char('n') => {
            if key_event.kind == KeyEventKind::Press && !app.is_mapspan_locked {
                app.jump_to_next_block();
            }
        }

        KeyCode::Char('N') => {
            if key_event.kind == KeyEventKind::Press && !app.is_mapspan_locked {
                app.jump_to_prev_block();
            }
        }

        KeyCode::Left => {
            if key_event.kind == KeyEventKind::Press {
                app.map_highlight = Some(app.map_highlight.unwrap_or(0).saturating_sub(1));
            }
        }

        KeyCode::Right => {

            if key_event.kind == KeyEventKind::Press {
                app.map_highlight = Some(app.map_highlight.unwrap_or(0).saturating_add(1));
            }
        }

        KeyCode::Up => {
            if key_event.kind == KeyEventKind::Press {
                app.map_highlight = Some(app.map_highlight.unwrap_or(0).saturating_sub(app.row_length));
            }
        }

        KeyCode::Down => {
            if key_event.kind == KeyEventKind::Press {
                app.map_highlight = Some(app.map_highlight.unwrap_or(0).saturating_add(app.row_length));
            }
        }

        KeyCode::Char('k') => {
            if key_event.kind == KeyEventKind::Press {
                app.map_highlight = Some(app.map_highlight.unwrap_or(0).saturating_add(1));
            }
        }

        KeyCode::Char('K') => {
            if key_event.kind == KeyEventKind::Press {
                app.map_highlight = Some(app.map_highlight.unwrap_or(0).saturating_sub(app.row_length));
            }
        }

        KeyCode::Char('(') => {
            if key_event.kind == KeyEventKind::Press {
                app.left_width = max(app.left_width.saturating_sub(DEFAULT_BLOCK_SIZE as u16 * 3), DEFAULT_BLOCK_SIZE as u16 * 3);
                app.right_width = min(app.right_width + DEFAULT_BLOCK_SIZE as u16 * 3, 100);
                app.row_length = app.row_length.saturating_add(DEFAULT_BLOCK_SIZE * 3);
            }
        }

        KeyCode::Char(')') => {
            if key_event.kind == KeyEventKind::Press {
                app.left_width = min(app.left_width + DEFAULT_BLOCK_SIZE as u16 * 3, 100);
                app.right_width = max(app.right_width.saturating_sub(DEFAULT_BLOCK_SIZE as u16 * 3), 10);
                app.row_length = app.row_length.saturating_sub(DEFAULT_BLOCK_SIZE * 3).clamp(MIN_ROW_LENGTH, usize::MAX);
            }
        }

        KeyCode::Char('0') => {
            if key_event.kind == KeyEventKind::Press {
                app.row_length = app.row_length.saturating_add(DEFAULT_BLOCK_SIZE);
            }
        }

        KeyCode::Char('9') => {
            if key_event.kind == KeyEventKind::Press {
                app.row_length = app.row_length.saturating_sub(DEFAULT_BLOCK_SIZE).clamp(MIN_ROW_LENGTH, usize::MAX);
            }
        }

        KeyCode::Char('1') => {
            if key_event.kind == KeyEventKind::Press {
                app.mode = DEFAULT;
            }
        }

        KeyCode::Char('2') => {
            if key_event.kind == KeyEventKind::Press {
                app.mode = STACKTRACE;
            }
        }

        // Other handlers you could add here.
        _ => {}
    }
    Ok(())
}
