use std::cmp::min;
use std::collections::HashMap;
use ratatui::{layout::Alignment, style::{Color, Style}, widgets::{Block, BorderType, Borders, Paragraph, canvas::*}, Frame};
use ratatui::prelude::{Constraint, Direction, Layout, Rect};

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
    draw_graph(app, left_inner_layout[0], frame, graph_data);
    
    let map_data;
    match app.graph_highlight {
        None => {
            map_data = app.damselfly_viewer.get_latest_map_state();
        }
        Some(highlight) => {
            let span = app.damselfly_viewer.get_span();
            map_data = &app.damselfly_viewer.get_map_state(span.0 + highlight);
        }
    }
    
    draw_memorymap(app, right_inner_layout[0], frame, map_data);

    frame.render_widget(
        Paragraph::new(format!(
            "OPERATIONS: {}", app.damselfly_viewer.get_total_operations()
        ))
        .block(
            Block::default()
                .title("Template")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .style(Style::default())
        .alignment(Alignment::Center),
        left_inner_layout[1]
    )
}

fn draw_graph(app: &mut App, area: Rect, frame: &mut Frame, data: &[(f64, f64)]) {
    let canvas = Canvas::default()
        .block(Block::default()
            .title("MEMORY USAGE")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded))
        .x_bounds([0.0, 100.0])
        .y_bounds([0.0, 90.0])
        .paint(|ctx| {
            ctx.draw(&Points { coords: data, color: Color::Red });
            if let Some(mut highlight) = app.graph_highlight {
                highlight = min(highlight, data.len() - 1);
                let (x, y) = data[highlight];
                ctx.draw(&Points { coords: &[(x, y)], color: Color::White })
            }
        });
    frame.render_widget(canvas, area);
}

fn draw_memorymap(app: &mut App, area: Rect, frame: &mut Frame, map: &HashMap<usize, MemoryStatus>) {
    let canvas = Canvas::default()
        .block(Block::default()
            .title("MEMORY MAP")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded))
        .x_bounds([0.0, 100.0])
        .y_bounds([0.0, 90.0])
        .paint(|ctx| {
            for address in 0..crate::damselfly_viewer::DamselflyViewer::DEFAULT_
    });
    frame.render_widget(canvas, area);
}