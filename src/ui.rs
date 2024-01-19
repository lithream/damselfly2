use std::cmp::min;
use std::collections::HashMap;
use std::rc::Rc;
use ratatui::{layout::Alignment, style::{Color, Style}, widgets::{Block, BorderType, Borders, Paragraph, canvas::*}, Frame};
use ratatui::prelude::{Constraint, Direction, Layout, Rect, Stylize};
use ratatui::style::Styled;
use ratatui::widgets::{Cell, Row, Table};
use ratatui::widgets::block::Title;

use crate::app::App;
use crate::damselfly_viewer::consts::{DEFAULT_MEMORY_SIZE, DEFAULT_MEMORYSPAN, DEFAULT_ROW_LENGTH};
use crate::memory::MemoryStatus;

/// Renders the user interface widgets.
pub fn render(app: &mut App, frame: &mut Frame) {
    // This is where you add new widgets.
    // See the following resources:
    // - https://docs.rs/ratatui/latest/ratatui/widgets/index.html
    // - https://github.com/ratatui-org/ratatui/tree/master/examples
    let main_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(70)
        ])
        .split(frame.size());

    let left_inner_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(80),
            Constraint::Percentage(20),
        ])
        .split(main_layout[0]);
    let right_inner_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(80),
            Constraint::Percentage(20),
        ])
        .split(main_layout[1]);

    let mut graph_binding = app.damselfly_viewer.get_memory_usage_view();
    graph_binding.iter_mut().for_each(|point| point.1 *= app.graph_scale);
    let graph_data = graph_binding.as_slice();
    if let Some(highlight) = app.graph_highlight {
        app.graph_highlight = Some(min(highlight, graph_data.len() - 1));
    }
    draw_graph(app, &left_inner_layout, frame, graph_data);

    let (map_data, latest_address) = match app.graph_highlight {
        None => {
            app.damselfly_viewer.get_latest_map_state()
        }
        Some(graph_highlight) => {
            let span = app.damselfly_viewer.get_timespan();
            app.damselfly_viewer.get_map_state(span.0 + graph_highlight)
        }
    };

    draw_memorymap(app, &right_inner_layout, frame, &map_data, latest_address);
}

fn snap_memoryspan_to_latest_operation(app: &mut App, latest_address: usize) {
    let mut new_map_span = app.map_span;
    if latest_address >= app.map_span.1 {
        new_map_span.0 = latest_address.saturating_sub(DEFAULT_MEMORYSPAN / 2);
        new_map_span.1 = new_map_span.0 + DEFAULT_MEMORYSPAN;
        app.map_highlight = Some(latest_address);
    } else if latest_address < app.map_span.0 {
        new_map_span.1 = latest_address + DEFAULT_MEMORYSPAN / 2;
        new_map_span.0 = new_map_span.1.saturating_sub(DEFAULT_MEMORYSPAN);
        app.map_highlight = Some(latest_address);
    }
    app.map_span = new_map_span;
}

fn draw_graph(app: &mut App, area: &Rc<[Rect]>, frame: &mut Frame, data: &[(f64, f64)]) {
    if data.is_empty() { return; }
    let graph_highlight;
    if let Some(highlight) = app.graph_highlight {
        graph_highlight = min(highlight, data.len().saturating_sub(1));
    } else {
        graph_highlight = data.len().saturating_sub(1);
    }
    let canvas = Canvas::default()
        .block(Block::default()
            .title(Title::from(format!("MEMORY USAGE {:.1}", app.graph_scale)))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded))
        .x_bounds([0.0, 100.0])
        .y_bounds([0.0, 90.0])
        .paint(|ctx| {
            ctx.draw(&Points { coords: data, color: Color::Red });
            if !data.is_empty() {
                let (x, y) = data[graph_highlight];
                ctx.draw(&Points { coords: &[(x, y)], color: Color::White });
            }
        });
    frame.render_widget(canvas, area[0]);

    let true_x = app.damselfly_viewer.get_timespan().0 + graph_highlight;
    let true_y = data[graph_highlight].1 / app.graph_scale;

    frame.render_widget(
        Paragraph::new(format!(
            "OPERATIONS: {}\n\
            TIME      : {}\n\
            USAGE %   : {}\n", app.damselfly_viewer.get_total_operations(), true_x, true_y
        ))
            .block(
                Block::default()
                    .title("USAGE STATS")
                    .title_alignment(Alignment::Left)
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .style(Style::default())
            .alignment(Alignment::Left),
        area[1]
    );
}

fn draw_memorymap(app: &mut App, area: &Rc<[Rect]>, frame: &mut Frame, map: &HashMap<usize, MemoryStatus>, latest_address: usize) {
    if app.is_mapspan_locked {
        snap_memoryspan_to_latest_operation(app, latest_address);
    }
    let latest_operation = app.damselfly_viewer.get_operation_address_at_time()
    let grid = generate_rows(app.map_span, app.map_highlight, map);
    let widths = [Constraint::Length(1); DEFAULT_ROW_LENGTH];
    let table = Table::new(grid)
        .widths(&widths)
        .column_spacing(0)
        .block(Block::default()
            .title("MEMORY MAP")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded));
    frame.render_widget(table, area[0]);

    frame.render_widget(
        Paragraph::new(format!(
            "MAP HIGHLIGHT: {}\n", app.map_highlight.unwrap_or(0)
        ))
            .block(
                Block::default()
                    .title("MAP")
                    .title_alignment(Alignment::Left)
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .style(Style::default())
            .alignment(Alignment::Left),
        area[1]
    );
}

fn generate_rows(map_span: (usize, usize), map_highlight: Option<usize>, map: &HashMap<usize, MemoryStatus>) -> Vec<Row> {
    let mut address = map_span.0;
    let mut grid: Vec<Row> = Vec::new();
    let push_cell = |row: &mut Vec<Cell>, block_state: &MemoryStatus, force_bg: Option<Color>| {
        let mut bg = Color::Black;
        let mut fg = Color::White;
        let content;
        match block_state {
            MemoryStatus::Allocated(_) => {
                content = "x";
                fg = Color::Red;
            }
            MemoryStatus::PartiallyAllocated(_) => {
                content = "=";
                fg = Color::Yellow;
            },
            MemoryStatus::Free(_) => {
                content = "o";
                fg = Color::White;
            },
        }
        if let Some(force_bg) = force_bg {
            bg = force_bg;
        }
        row.push(Cell::from(content).style(Style::default().bg(bg).fg(fg)));
    };
    let push_cell_or_default = |current_row: &mut Vec<Cell>, map: &HashMap<usize, MemoryStatus>, address: usize, force_bg: Option<Color>| {
        if let Some(block_state) = map.get(&address) {
            push_cell(current_row, block_state, force_bg);
        } else if let Some(bg) = force_bg {
            current_row.push(Cell::from(".").style(Style::default().bg(bg).fg(Color::White)));
        } else {
            current_row.push(Cell::from(".").style(Style::default()));
        }
    };

    for _row in 0..(DEFAULT_MEMORY_SIZE / DEFAULT_ROW_LENGTH) {
        let mut current_row: Vec<Cell> = Vec::new();
        for _col in 0..DEFAULT_ROW_LENGTH {
            match map_highlight {
                None => {
                    push_cell_or_default(&mut current_row, map, address, None);
                }
                Some(map_highlight) => {
                    if address == map_highlight {
                        push_cell_or_default(&mut current_row, map, address, Some(Color::Green));
                    } else {
                        push_cell_or_default(&mut current_row, map, address, None);
                    }
                }
            }
            address += 1;
        }
        grid.push(Row::new(current_row));
    }
    grid
}
