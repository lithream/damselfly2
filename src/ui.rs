use std::cmp::min;
use std::collections::HashMap;
use std::rc::Rc;
use ratatui::{layout::Alignment, style::{Color, Style}, widgets::{Block, BorderType, Borders, Paragraph, canvas::*}, Frame};
use ratatui::prelude::{Constraint, Direction, Layout, Rect, Stylize};
use ratatui::widgets::{Cell, Row, Table};
use ratatui::widgets::block::Title;

use crate::app::App;
use crate::damselfly_viewer::consts::{DEFAULT_MEMORY_SIZE, DEFAULT_ROW_LENGTH};
use crate::memory::MemoryStatus;

enum GridState {
    Allocated,
    PartiallyAllocated,
    Free
}

/// Renders the user interface widgets.
pub fn render(app: &mut App, frame: &mut Frame) {
    // This is where you add new widgets.
    // See the following resources:
    // - https://docs.rs/ratatui/latest/ratatui/widgets/index.html
    // - https://github.com/ratatui-org/ratatui/tree/master/examples
    let main_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50)
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

    let map_data: HashMap<usize, MemoryStatus>;
    match app.graph_highlight {
        None => {
            map_data = app.damselfly_viewer.get_latest_map_state().clone();
        }
        Some(highlight) => {
            let span = app.damselfly_viewer.get_timespan();
            map_data = app.damselfly_viewer.get_map_state(span.0 + highlight).clone();
        }
    }

    draw_memorymap(app, &right_inner_layout, frame, &map_data);
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
    )
}

fn draw_memorymap(app: &mut App, area: &Rc<[Rect]>, frame: &mut Frame, map: &HashMap<usize, MemoryStatus>) {
    let grid = generate_rows(app.map_highlight, map);
    let widths = [Constraint::Length(1); DEFAULT_ROW_LENGTH];
    let table = Table::new(grid)
        .widths(&widths)
        .column_spacing(0)
        .block(Block::default()
            .title("MEMORY MAP")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded))
        .highlight_style(Style::new().reversed())
        .highlight_symbol(">>");
    frame.render_stateful_widget(table, area[0], &mut app.table_state);

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
    )
}

fn generate_rows(map_highlight: Option<usize>, map: &HashMap<usize, MemoryStatus>) -> Vec<Row> {
    let mut address: usize = 0;
    let mut grid: Vec<Row> = Vec::new();
    let push_cell = |row: &mut Vec<Cell>, block_state: &MemoryStatus| {
        match block_state {
            MemoryStatus::Allocated(_) => {
                row.push(Cell::from("x").style(Style::default().red()));
            }
            MemoryStatus::PartiallyAllocated(_) => {
                row.push(Cell::from("=").style(Style::default().yellow()));
            }
            MemoryStatus::Free(_) => {
                row.push(Cell::from("o").style(Style::default().gray()));
            }
        }
    };
    let push_cell_or_default = |current_row: &mut Vec<Cell>, map: &HashMap<usize, MemoryStatus>, address: usize| {
        if let Some(block_state) = map.get(&address) {
            push_cell(current_row, block_state);
        } else {
            current_row.push(Cell::from("o").style(Style::default().gray()));
        }
    };

    for row in 0..(DEFAULT_MEMORY_SIZE / DEFAULT_ROW_LENGTH) {
        let mut current_row: Vec<Cell> = Vec::new();
        for col in 0..DEFAULT_ROW_LENGTH {
            let style = Style::default().gray();
            match map_highlight {
                None => {
                    push_cell_or_default(&mut current_row, map, address);
                }
                Some(map_highlight) => {
                    if address == map_highlight {
                        current_row.push(Cell::from("#").style(Style::default().green()));
                    } else {
                        push_cell_or_default(&mut current_row, map, address);
                    }
                }
            }
            address += 1;
        }
        grid.push(Row::new(current_row));
    }
    grid
}
