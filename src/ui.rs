use std::cmp::{min};
use std::collections::HashMap;
use std::rc::Rc;
use ratatui::{layout::Alignment, style::{Color, Style}, widgets::{Block, BorderType, Borders, Paragraph, canvas::*}, Frame};
use ratatui::prelude::{Constraint, Direction, Layout, Rect, Stylize};
use ratatui::style::Styled;
use ratatui::text::Span;
use ratatui::widgets::{Cell, Row, Table, Wrap};
use ratatui::widgets::block::Title;

use crate::app::App;
use crate::damselfly_viewer::consts::{DEFAULT_BLOCK_SIZE, DEFAULT_MEMORY_SIZE, DEFAULT_MEMORYSPAN, DEFAULT_ROW_LENGTH, DEFAULT_TIMESPAN};
use crate::map_manipulator::MapManipulator;
use crate::memory::{MemoryStatus, MemoryUpdate};

/// Renders the user interface widgets.
pub fn render(app: &mut App, frame: &mut Frame) {
    // This is where you add new widgets.
    // See the following resources:
    // - https://docs.rs/ratatui/latest/ratatui/widgets/index.html
    // - https://github.com/ratatui-org/ratatui/tree/master/examples
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(app.up_height),
            Constraint::Percentage(app.down_height)
        ])
        .split(frame.size());

    let up_inner_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(app.left_width),
            Constraint::Percentage(app.right_width),
        ])
        .split(main_layout[0]);
    let down_inner_layout = Layout::default()
        .direction(Direction::Horizontal)
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
    draw_graph(app, &up_inner_layout, frame, graph_data);

    let (map_data, latest_operation) = {
        match app.graph_highlight {
            None => {
                let map_state = app.damselfly_viewer.get_latest_map_state();
                let map = map_state.0.clone();
                let operation = map_state.1.cloned();
                (map, operation)
            }
            Some(graph_highlight) => {
                let span = app.damselfly_viewer.get_timespan();
                let map_state = app.damselfly_viewer.get_map_state(span.0 + graph_highlight);
                //let map_state = app.damselfly_viewer.get_map_state(span.0 + graph_highlight.clone());
                let map = map_state.0.clone();
                let operation = map_state.1.cloned();
                (map, operation)
            }
        }
    };

    draw_memorymap(app, &up_inner_layout, &down_inner_layout, frame, &map_data, latest_operation);
}

fn snap_memoryspan_to_latest_operation(app: &mut App, latest_address: usize) {
    let mut new_map_span = app.map_span;
    let relative_address = MapManipulator::scale_address_down(latest_address);
    let address_of_row = MapManipulator::get_address_of_row(relative_address);
    if relative_address >= app.map_span.1 {
        new_map_span.0 = address_of_row.saturating_sub(DEFAULT_MEMORYSPAN / 2);
        new_map_span.1 = new_map_span.0 + DEFAULT_MEMORYSPAN;
    } else if relative_address < app.map_span.0 {
        new_map_span.1 = relative_address + DEFAULT_MEMORYSPAN / 2;
        new_map_span.0 = new_map_span.1.saturating_sub(DEFAULT_MEMORYSPAN);
    }
    app.map_highlight = Some(relative_address);
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

    let true_x = app.damselfly_viewer.get_timespan().0 + graph_highlight;
    let true_y = data[graph_highlight].1 / app.graph_scale;
    let canvas = Canvas::default()
        .block(Block::default()
            .title(Title::from(format!("[ZOOM: {:.1}] [OPERATION: {} / {}] [USAGE: {:.2}]",
                                       app.graph_scale, true_x, app.damselfly_viewer.get_total_operations(),
                                        true_y)))
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

fn draw_memorymap(app: &mut App, map_area: &Rc<[Rect]>, stats_area: &Rc<[Rect]>, frame: &mut Frame, map: &HashMap<usize, MemoryStatus>, latest_operation: Option<MemoryUpdate>) {
    let right_inner_layout_upper = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(100),
            Constraint::Percentage(0),
        ])
        .split(area[0]);

    let latest_address = match latest_operation {
        None => 0,
        Some(operation) => match operation {
            MemoryUpdate::Allocation(address, _, _) => address,
            MemoryUpdate::Free(address, _) => address,
        }
    };
    if app.is_mapspan_locked {
        snap_memoryspan_to_latest_operation(app, latest_address);
        app.map_highlight = Some(latest_address / DEFAULT_BLOCK_SIZE);
    }

    let grid = generate_rows(DEFAULT_MEMORY_SIZE / DEFAULT_ROW_LENGTH, app.map_span, app.map_highlight, map);
    let widths = [Constraint::Length(1); DEFAULT_ROW_LENGTH];
    let locked_status;
    let title_style;
    match app.is_mapspan_locked {
        true => {
            locked_status = "LOCKED";
            title_style = Style::default().red();
        },
        false => {
            locked_status = "UNLOCKED";
            title_style = Style::default().green();
        }
    };
    let table = Table::new(grid)
        .widths(&widths)
        .column_spacing(0)
        .block(Block::default()
            .title(format!("MEMORY MAP [{:x}] [VIEW: {locked_status}]",
                           MapManipulator::scale_address_up(app.map_highlight.unwrap_or(0))))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title_style(title_style));
    frame.render_widget(table, right_inner_layout_upper[0]);

    let callstack = match map.get(&app.map_highlight.unwrap_or(0)) {
        None => "",
        Some(memory_status) => {
            match memory_status {
                MemoryStatus::Allocated(_, callstack) => callstack,
                MemoryStatus::PartiallyAllocated(_, callstack) => callstack,
                MemoryStatus::Free(callstack) => callstack
            }
        }
    };

    frame.render_widget(
        Paragraph::new(callstack.to_string())
            .block(
                Block::default()
                    .title("MAP")
                    .title_alignment(Alignment::Left)
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .style(Style::default())
            .alignment(Alignment::Left)
            .wrap(Wrap::default()),
        right_inner_layout_bottom[0]
    );

    let operation_count;
    let operation_list;
    if app.damselfly_viewer.is_timespan_locked() {
        operation_count = app.damselfly_viewer.get_total_operations();
        operation_list = app.damselfly_viewer
            .get_operation_log_span(operation_count.saturating_sub(7), operation_count);
    } else {
        let mut timespan = app.damselfly_viewer.get_timespan();
        timespan.0 = timespan.0.saturating_sub(7);
        match app.graph_highlight {
            None => {}
            Some(highlight) => timespan.1 = timespan.0 + highlight,
        }
        operation_list = app.damselfly_viewer
            .get_operation_log_span(timespan.0, timespan.1);
    }
    let mut rows = Vec::new();
    for operation in operation_list.iter().rev() {
        let style = match operation {
            MemoryUpdate::Allocation(_, size, _) => {
                if *size < DEFAULT_BLOCK_SIZE {
                    Style::default().yellow()
                } else {
                    Style::default().red()
                }
            },
            MemoryUpdate::Free(_, _) => Style::default().gray(),
        };
        rows.push(Row::new(vec![operation.to_string()]).set_style(style));
    }
    let widths = [
        Constraint::Percentage(100),
    ];
    let table = Table::new(rows).widths(&widths[..])
        .block(Block::default()
            .title("OPERATIONS")
            .title_alignment(Alignment::Left)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded));

    frame.render_widget(table, right_inner_layout_bottom[1]);
}

fn generate_rows(rows: usize, map_span: (usize, usize), map_highlight: Option<usize>, map: &HashMap<usize, MemoryStatus>) -> Vec<Row> {
    let mut address = map_span.0;
    let mut grid: Vec<Row> = Vec::new();
    let push_cell = |row: &mut Vec<Cell>, block_state: &MemoryStatus, force_bg: Option<Color>| {
        let mut bg = Color::Black;
        let fg;
        let content;
        match block_state {
            MemoryStatus::Allocated(..) => {
                content = "x";
                fg = Color::Red;
            }
            MemoryStatus::PartiallyAllocated(..) => {
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

    for _row in 0..rows {
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

#[cfg(test)]
mod tests {

}