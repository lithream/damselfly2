/*
use crate::damselfly::map_manipulator;
use crate::app::Mode;
use std::rc::Rc;
use std::time::Instant;
use ratatui::{layout::Alignment, style::{Color, Style}, widgets::{Block, BorderType, Borders, Paragraph, canvas::*}, Frame};
use ratatui::prelude::{Constraint, Direction, Layout, Rect, Stylize};
use ratatui::style::Styled;
use ratatui::widgets::{Cell, Row, Table, Wrap};
use ratatui::widgets::block::Title;

use crate::app::App;
use crate::damselfly::consts::{DEFAULT_BLOCK_SIZE, DEFAULT_MEMORYSPAN, DEFAULT_TIMESPAN, GRAPH_VERTICAL_SCALE_OFFSET};
use crate::damselfly::memory_structs::{MemoryStatus, MemoryUpdate, NoHashMap};

/// Renders the user interface widgets.
pub fn render(app: &mut App, frame: &mut Frame) {
    // This is where you add new widgets.
    // See the following resources:
    // - https://docs.rs/ratatui/latest/ratatui/widgets/index.html
    // - https://github.com/ratatui-org/ratatui/tree/master/examples
    match app.mode {
        Mode::DEFAULT => {
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
                    Constraint::Percentage(app.up_left_width),
                    Constraint::Percentage(app.up_middle_width),
                    Constraint::Percentage(app.up_right_width),
                ])
                .split(main_layout[0]);
            let down_inner_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(80),
                    Constraint::Percentage(20),
                ])
                .split(main_layout[1]);

            let start = Instant::now();
            let graph_data = get_graph_data(app);
            draw_graph(app, &up_inner_layout[0], frame, graph_data);

            let (map_data, latest_operation) = get_map_and_latest_op(app);
            draw_memorymap(app, &up_inner_layout[1], &down_inner_layout, frame, &map_data, latest_operation);

            let blocks_data = get_blocks_data(app);
            let duration = start.elapsed();
            draw_stats(&up_inner_layout[2], frame, &blocks_data, duration.as_millis() as usize);
        }
        Mode::STACKTRACE => {
            let main_layout = Layout::default()
                .constraints([
                    Constraint::Percentage(100)
                ]).split(frame.size());
            let (map_data, _) = get_map_and_latest_op(app);
            draw_stacktrace(&app, &main_layout, frame, &map_data);
        }
    }
}

fn get_map_and_latest_op(app: &mut App) -> (NoHashMap<usize, MemoryStatus>, Option<MemoryUpdate>) {
    let (map_data, latest_operation) = {
            let map_state =
                app.damselfly_controller.get_current_map_state();
            let map = map_state.0;
            let operation = map_state.1.cloned();
            (map, operation)
    };
    (map_data, latest_operation)
}

fn get_graph_data(app: &mut App) -> Vec<[f64; 2]> {
    let mut graph_binding = app.damselfly_controller.get_current_memory_usage_graph();
    graph_binding.iter_mut().for_each(|point| {
        point[1] *= app.graph_scale;
        point[1] /= GRAPH_VERTICAL_SCALE_OFFSET;
    });
    let graph_data = graph_binding.as_slice();
    Vec::from(graph_data)
}

fn get_blocks_data(app: &mut App) -> usize {
    app.damselfly_controller.get_current_memory_usage().blocks
}

fn snap_memoryspan_to_latest_operation(app: &mut App, latest_address: usize) {
    app.damselfly_controller.snap_memoryspan_to_address(latest_address);
}

fn draw_graph(app: &mut App, area: &Rect, frame: &mut Frame, data: Vec<(f64, f64)>) {
    if data.is_empty() { return; }

    let true_x = app.damselfly_controller.get_graph_highlight_absolute();
    let relative_x = app.damselfly_controller.graph_highlight;
    let true_y = (data[relative_x].1 / app.graph_scale) * GRAPH_VERTICAL_SCALE_OFFSET;
    let canvas = Canvas::default()
        .block(Block::default()
            .title(Title::from(format!("[ZOOM: {:.1}] [OPERATION: {} / {}] [USAGE: {:.2}]",
                                       app.graph_scale, true_x, app.damselfly_controller.viewer.get_total_operations() - 1,
                                        true_y)))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded))
        .x_bounds([0.0, DEFAULT_TIMESPAN as f64])
        .y_bounds([0.0, 90.0])
        .paint(|ctx| {
            ctx.draw(&Points { coords: &data, color: Color::Red });
            if !data.is_empty() {
                let (x, y) = data[relative_x];
                ctx.draw(&Points { coords: &[(x, y)], color: Color::White });
            }
        });
    frame.render_widget(canvas, *area);
}

fn draw_memorymap(app: &mut App, map_area: &Rect, stats_area: &Rc<[Rect]>, frame: &mut Frame, map: &NoHashMap<usize, MemoryStatus>, latest_operation: Option<MemoryUpdate>) {
    let latest_address = match latest_operation {
        None => 0,
        Some(operation) => match operation {
            MemoryUpdate::Allocation(address, _, _) => address,
            MemoryUpdate::Free(address, _) => address,
        }
    };
    if !app.damselfly_controller.memoryspan_freelook {
        snap_memoryspan_to_latest_operation(app, latest_address);
    }

    let grid = generate_rows(DEFAULT_MEMORYSPAN / app.row_length, app.row_length, app.damselfly_controller.memory_span,
                             app.damselfly_controller.map_highlight, map);
    let widths = vec![Constraint::Length(1); app.row_length];
    let locked_status;
    let title_style;
    match app.damselfly_controller.memoryspan_freelook {
        false => {
            locked_status = "LOCKED";
            title_style = Style::default().red();
        },
        true => {
            locked_status = "UNLOCKED";
            title_style = Style::default().green();
        }
    };
    let address_bounds = app.damselfly_controller.viewer.get_address_bounds();
    let table = Table::new(grid)
        .widths(&widths)
        .column_spacing(0)
        .block(Block::default()
            .title(format!("MEMORY MAP [{:x}] [VIEW: {locked_status}] [{:x} - {:x}]",
                           map_manipulator::logical_to_absolute(app.damselfly_controller.map_highlight),
                           (address_bounds.0),
                           (address_bounds.1))
            )
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title_style(title_style));
    frame.render_widget(table, *map_area);

    draw_stacktrace(&app, stats_area, frame, map);

    let operation_list = app.damselfly_controller.get_current_operation_log();
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

    frame.render_widget(table, stats_area[1]);
}

fn draw_stacktrace(app: &&mut App, stats_area: &Rc<[Rect]>, frame: &mut Frame, map: &NoHashMap<usize, MemoryStatus>) {
    let callstack = match map.get(&app.damselfly_controller.map_highlight) {
        None => "",
        Some(memory_status) => {
            match memory_status {
                MemoryStatus::Allocated(_, _, callstack) => callstack,
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
        stats_area[0]
    );
}

fn draw_stats(area: &Rect, frame: &mut Frame, blocks_data: &usize, time: usize) {
    frame.render_widget(
        Paragraph::new(format!("Blocks: {blocks_data} TIME: {time}"))
            .block(
                Block::default()
                    .title("STATS")
                    .title_alignment(Alignment::Left)
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .style(Style::default())
            .alignment(Alignment::Left)
            .wrap(Wrap::default()),
        *area
    );
}

fn generate_rows(rows: usize, row_length: usize, map_span: (usize, usize), map_highlight: usize, map: &NoHashMap<usize, MemoryStatus>) -> Vec<Row> {
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
    let push_cell_or_default = |current_row: &mut Vec<Cell>, map: &NoHashMap<usize, MemoryStatus>, address: usize, force_bg: Option<Color>| {
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
        for _col in 0..row_length {
            if address == map_highlight {
                push_cell_or_default(&mut current_row, map, address, Some(Color::Green));
            } else {
                push_cell_or_default(&mut current_row, map, address, None);
            }
            address += 1;
        }
        grid.push(Row::new(current_row));
    }
    grid
}
 */