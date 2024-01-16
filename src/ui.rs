use std::cmp::min;
use std::ops::Deref;
use ratatui::{layout::Alignment, style::{Color, Style}, widgets::{Block, BorderType, Borders, Paragraph, canvas::*}, Frame, symbols};
use ratatui::prelude::{Constraint, Direction, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::text::Span;
use ratatui::widgets::{Axis, Chart, Dataset};

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

    let binding = app.damselfly_viewer.get_memory_usage_view();
    let data = binding.as_slice();
    if let Some(highlight) = app.graph_highlight {
        app.graph_highlight = Some(min(highlight, data.len() - 1));
    }
    draw_graph(app, left_inner_layout[0], frame, data);
    //draw_mmap(app, right_inner_layout[0], frame, data);

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

fn draw_mmap(app: &mut App, area: Rect, frame: &mut Frame, data: &[((f64, f64), MemoryStatus)]) {
    let canvas = Canvas::default()
        .block(Block::default()
            .title("MEMORY MAP")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded))
        .x_bounds([0.0, 100.0])
        .y_bounds([0.0, 90.0])
        .paint(|ctx| {
        for point in data {
            match point.1 {
                MemoryStatus::Allocated(_) => {
                    ctx.draw(&Points{ coords: &[point.0], color: Color::Red });
                }
                MemoryStatus::PartiallyAllocated(_) => {
                    ctx.draw(&Points{ coords: &[point.0], color: Color::Yellow });
                }
                MemoryStatus::Free(_) => {
                    ctx.draw(&Points{ coords: &[point.0], color: Color::Green });
                }
            }
        }
    });
    frame.render_widget(canvas, area);
}