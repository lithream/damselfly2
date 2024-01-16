use ratatui::{layout::Alignment, style::{Color, Style}, widgets::{Block, BorderType, Borders, Paragraph, canvas::*}, Frame, symbols};
use ratatui::prelude::{Constraint, Direction, Layout};
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
    let binding = app.damselfly_viewer.get_memory_usage_view();
    let data = binding.as_slice();

    let ctx = Context::new(
        100,
        100,
        [-180.0, 180.0],
        [-90.0, 90.0],
        symbols::Marker::Braille
    );

    let canvas = Canvas::default()
        .block(Block::default().title("Canvas").borders(Borders::ALL))
        .x_bounds([0.0, 100.0])
        .y_bounds([0.0, 90.0])
        .paint(|ctx| {
            ctx.draw(&Points { coords: data, color: Color::Red });
            if let Some(highlight) = app.highlight {
                let (x, y) = data[highlight];
                ctx.draw(&Points { coords: &[(x, y)], color: Color::White })
            }
        });
    frame.render_widget(canvas, main_layout[0]);

    frame.render_widget(
        Paragraph::new(format!(
            "This is a tui template.\n\
                Press `Esc`, `Ctrl-C` or `q` to stop running.\n\
                Press left and right to increment and decrement the counter respectively.\n\
                Counter: {}",
            app.damselfly_viewer.get_memory_usage().memory_used_absolute
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
