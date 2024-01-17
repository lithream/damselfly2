use std::cmp::min;
use std::collections::HashMap;
use std::rc::Rc;
use ratatui::{layout::Alignment, style::{Color, Style}, widgets::{Block, BorderType, Borders, Paragraph, canvas::*}, Frame};
use ratatui::prelude::{Constraint, Direction, Layout, Rect, Stylize};
use ratatui::widgets::{Row, Table};

use crate::app::App;
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

    let graph_binding = app.damselfly_viewer.get_memory_usage_view();
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

    draw_memorymap(app, right_inner_layout[0], frame, &map_data);



}

fn draw_graph(app: &mut App, area: &Rc<[Rect]>, frame: &mut Frame, data: &[(f64, f64)]) {
    let graph_highlight;
    if !data.is_empty() {
        if let Some(highlight) = app.graph_highlight {
            graph_highlight = min(highlight, data.len().saturating_sub(1));
        } else {
            graph_highlight = data.len().saturating_sub(1);
        }
    }
    let canvas = Canvas::default()
        .block(Block::default()
            .title("MEMORY USAGE")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded))
        .x_bounds([0.0, 100.0])
        .y_bounds([0.0, 90.0])
        .paint(|ctx| {
            ctx.draw(&Points { coords: data, color: Color::Red });
            if !data.is_empty() {
                if let Some(mut highlight) = app.graph_highlight {
                    highlight = min(highlight, data.len().saturating_sub(1));
                    let (x, y) = data[highlight];
                    ctx.draw(&Points { coords: &[(x, y)], color: Color::White })
                } else {
                    let highlight = data.len().saturating_sub(1);
                    let (x, y) = data[highlight];
                    ctx.draw(&Points { coords: &[(x, y)], color: Color::White })
                }
            }
        });
    frame.render_widget(canvas, area[0]);

    let mut true_x = 0;
    let mut y = 0.0;
    if let Some(graph_highlight) = app.graph_highlight {
        true_x = app.damselfly_viewer.get_timespan().0 + graph_highlight;
        y = data[graph_highlight].1;
    }

    frame.render_widget(
        Paragraph::new(format!(
            "OPERATIONS: {}\n\
            TIME      : {}\n\
            USAGE %   : {}\n", app.damselfly_viewer.get_total_operations(), true_x, y
        ))
            .block(
                Block::default()
                    .title("USAGE GRAPH")
                    .title_alignment(Alignment::Left)
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .style(Style::default())
            .alignment(Alignment::Left),
        area[1]
    )
}

fn draw_memorymap(app: &mut App, area: Rect, frame: &mut Frame, map: &HashMap<usize, MemoryStatus>) {
    let rows = [Row::new(vec!["Cell1", "Cell2", "Cell3"])];
    let widths = [
        Constraint::Length(5),
        Constraint::Length(5),
        Constraint::Length(10),
    ];
    let table = Table::new(rows)
        .widths(&widths)
        .column_spacing(1)
        .style(Style::new().blue())
        .header(
            Row::new(vec!["Col1", "Col2", "Col3"])
                .style(Style::new().bold())
                .bottom_margin(1)
        )
        .block(Block::default().title("Table"))
        .highlight_style(Style::new().reversed())
        .highlight_symbol(">>");
    /*
    let canvas = Canvas::default()
        .block(Block::default()
            .title("MEMORY MAP")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded))
        .x_bounds([0.0, 100.0])
        .y_bounds([0.0, 90.0])
        .paint(|ctx| {
            for address in 0..DEFAULT_MEMORY_SIZE {


            }
    });
     */
    frame.render_widget(table, area);
}