use ratatui::{layout::Alignment, style::{Color, Style}, widgets::{Block, BorderType, Borders, Paragraph}, Frame, symbols};
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
            Constraint::Percentage(100)
        ])
        .split(frame.size());
    let memory_usage_view = app.damselfly_viewer.get_memory_usage_view();

    let dataset = Dataset::default()
        .name("Memory usage")
        .marker(symbols::Marker::Dot)
        .style(Style::default().fg(Color::Cyan))
        .data(memory_usage_view);

    let graph_chart = Chart::new(vec![dataset])
        .block(
            Block::default()
                .title("Memory usage".cyan().bold())
                .borders(Borders::ALL),
        )
        .x_axis(
            Axis::default()
                .title("Memory operations")
                .style(Style::default().fg(Color::Gray))
                .bounds([-20.0, 20.0])
        )
        .y_axis(
            Axis::default()
                .title("Memory usage")
                .style(Style::default().fg(Color::Gray))
                .labels(vec![Span::raw("-20"), Span::raw("0"), Span::raw("0")])
                .bounds([-20.0, 20.0])
        );
    frame.render_widget(graph_chart, main_layout[0]);

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
        frame.size(),
    )
}
