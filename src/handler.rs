use std::cmp::{max, min};
use crate::app::{App, AppResult};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crate::app::Mode::{DEFAULT, STACKTRACE};
use crate::damselfly::consts::{DEFAULT_BLOCK_SIZE, DEFAULT_MEMORYSPAN, DEFAULT_TIMESPAN, MIN_ROW_LENGTH};

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
                app.damselfly_controller.unlock_timespan();
            }
        }

        KeyCode::Char(':') => {
            if key_event.kind == KeyEventKind::Press {
                app.damselfly_controller.lock_timespan();
            }
        }

        KeyCode::Char('\'') => {
            if key_event.kind == KeyEventKind::Press {
                app.damselfly_controller.memoryspan_freelook = !app.damselfly_controller.memoryspan_freelook;
            }
        }

        KeyCode::Char('H') => {
            if key_event.kind == KeyEventKind::Press {
                app.damselfly_controller.shift_timespan_left(1);
            }
        }

        KeyCode::Char('L') => {
            if key_event.kind == KeyEventKind::Press {
                app.damselfly_controller.shift_timespan_right(1);
            }
        }

        KeyCode::Char('h') => {
            if key_event.kind == KeyEventKind::Press {
                app.damselfly_controller.shift_graph_highlight_left();
            }
        }

        KeyCode::Char('l') => {
            if key_event.kind == KeyEventKind::Press {
                app.damselfly_controller.shift_graph_highlight_right();
            }
        }

        KeyCode::Char('i') => {
            if key_event.kind == KeyEventKind::Press {
                app.damselfly_controller.graph_highlight = 0;
            }
        }

        KeyCode::Char('o') => {
            if key_event.kind == KeyEventKind::Press {
                let span = app.damselfly_controller.get_timespan();
                app.damselfly_controller.graph_highlight = span.1 - span.0 - 1;
            }
        }

        KeyCode::Char('[') => {
            if key_event.kind == KeyEventKind::Press {
                app.damselfly_controller.unlock_timespan();
                app.damselfly_controller.shift_timespan_to_beginning();
            }
        }

        KeyCode::Char(']') => {
            if key_event.kind == KeyEventKind::Press {
                app.damselfly_controller.lock_timespan();
                app.damselfly_controller.unlock_timespan();
                app.damselfly_controller.shift_timespan_to_end();
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
                app.damselfly_controller.map_highlight = app.damselfly_controller.map_highlight.saturating_sub(1);
            }
        }

        KeyCode::Char('J') => {
            if key_event.kind == KeyEventKind::Press {
                app.damselfly_controller.map_highlight = app.damselfly_controller.map_highlight.saturating_add(app.row_length);
            }
        }

        KeyCode::PageUp => {
            if key_event.kind == KeyEventKind::Press {
                if key_event.modifiers == KeyModifiers::SHIFT {
                    app.damselfly_controller.memory_span.0 = app.damselfly_controller.memory_span.0.saturating_sub(DEFAULT_MEMORYSPAN);
                    app.damselfly_controller.memory_span.1 = app.damselfly_controller.memory_span.1.saturating_sub(DEFAULT_MEMORYSPAN);
                    app.damselfly_controller.map_highlight = app.damselfly_controller.map_highlight.saturating_sub(DEFAULT_MEMORYSPAN);
                } else {
                    app.damselfly_controller.memory_span.0 = app.damselfly_controller.memory_span.0.saturating_sub(app.row_length);
                    app.damselfly_controller.memory_span.1 = app.damselfly_controller.memory_span.1.saturating_sub(app.row_length);
                    app.damselfly_controller.map_highlight = app.damselfly_controller.map_highlight.saturating_sub(app.row_length);
                }
            }
        }

        KeyCode::PageDown => {
            if key_event.kind == KeyEventKind::Press {
                if key_event.modifiers == KeyModifiers::SHIFT {
                    app.damselfly_controller.memory_span.0 = app.damselfly_controller.memory_span.0.saturating_add(DEFAULT_MEMORYSPAN);
                    app.damselfly_controller.memory_span.1 = app.damselfly_controller.memory_span.1.saturating_add(DEFAULT_MEMORYSPAN);
                    app.damselfly_controller.map_highlight = app.damselfly_controller.map_highlight.saturating_add(DEFAULT_MEMORYSPAN);
                }
                app.damselfly_controller.memory_span.0 = app.damselfly_controller.memory_span.0.saturating_add(app.row_length);
                app.damselfly_controller.memory_span.1 = app.damselfly_controller.memory_span.1.saturating_add(app.row_length);
                app.damselfly_controller.map_highlight = app.damselfly_controller.map_highlight.saturating_add(app.row_length);
            }
        }

        KeyCode::Left => {
            if key_event.kind == KeyEventKind::Press {
                app.damselfly_controller.map_highlight = app.damselfly_controller.map_highlight.saturating_sub(1);
            }
        }

        KeyCode::Right => {

            if key_event.kind == KeyEventKind::Press {
                app.damselfly_controller.map_highlight = app.damselfly_controller.map_highlight.saturating_add(1);
            }
        }

        KeyCode::Up => {
            if key_event.kind == KeyEventKind::Press {
                app.damselfly_controller.map_highlight = app.damselfly_controller.map_highlight.saturating_sub(app.row_length);
            }
        }

        KeyCode::Down => {
            if key_event.kind == KeyEventKind::Press {
                app.damselfly_controller.map_highlight = app.damselfly_controller.map_highlight.saturating_add(app.row_length);
            }
        }

        KeyCode::Char('k') => {
            if key_event.kind == KeyEventKind::Press {
                app.damselfly_controller.map_highlight = app.damselfly_controller.map_highlight.saturating_add(1);
            }
        }

        KeyCode::Char('K') => {
            if key_event.kind == KeyEventKind::Press {
                app.damselfly_controller.map_highlight = app.damselfly_controller.map_highlight.saturating_sub(app.row_length);
            }
        }

        KeyCode::Char('(') => {
            if key_event.kind == KeyEventKind::Press {
                app.up_left_width = max(app.up_left_width.saturating_sub(DEFAULT_BLOCK_SIZE as u16 * 3), DEFAULT_BLOCK_SIZE as u16 * 3);
                app.up_right_width = min(app.up_right_width + DEFAULT_BLOCK_SIZE as u16 * 3, 100);
                app.row_length = app.row_length.saturating_add(DEFAULT_BLOCK_SIZE * 3);
            }
        }

        KeyCode::Char(')') => {
            if key_event.kind == KeyEventKind::Press {
                app.up_left_width = min(app.up_left_width + DEFAULT_BLOCK_SIZE as u16 * 3, 100);
                app.up_right_width = max(app.up_right_width.saturating_sub(DEFAULT_BLOCK_SIZE as u16 * 3), 10);
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
