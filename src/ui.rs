use std::cmp::min;
use ratatui::{layout::Alignment, style::{Color, Style}, widgets::{Block, BorderType, Borders, Paragraph, canvas::*}, Frame, symbols};
use ratatui::prelude::{Constraint, Direction, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::text::Span;
use ratatui::widgets::{Axis, Chart, Dataset};

use crate::app::App;

/// Renders the user interface widgets.
pub fn render(app: &mut App, frame: &mut Frame) {
    // This is where you add new widgets.
    // See the following resources:
    // - https://docs.rs/ratatui/latest/ratatui/widgets/index.html
    // - https://github.com/ratatui-org/ratatui/tree/master/examples
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(80),
            Constraint::Percentage(20)
        ])
        .split(frame.size());

    draw_graph(app, main_layout[0], frame);

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
        .style(Style::default().fg(Color::Cyan).bg(Color::Black))
        .alignment(Alignment::Center),
        main_layout[1]
    )
}

fn draw_graph(app: &mut App, area: Rect, frame: &mut Frame) {
    let binding = app.damselfly_viewer.get_memory_usage_view();
    let data = binding.as_slice();

    let canvas = Canvas::default()
        .block(Block::default()
            .title("MEMORY USAGE")
            .borders(Borders::ALL)
            .border_type(BorderType::Double))
        .x_bounds([0.0, 100.0])
        .y_bounds([0.0, 90.0])
        .paint(|ctx| {
            ctx.draw(&Points { coords: data, color: Color::Red });
            if let Some(mut highlight) = app.highlight {
                highlight = min(highlight, data.len() - 1);
                let (x, y) = data[highlight];
                ctx.draw(&Points { coords: &[(x, y)], color: Color::White })
            }
        });
    frame.render_widget(canvas, area);
}

