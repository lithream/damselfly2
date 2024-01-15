use ratatui::prelude::{Style, Stylize};
use ratatui::widgets::{Axis, Block, Chart, Dataset};

enum DataType {
    MemoryUsage,
}

struct GraphWidget<'a> {
    datasets: Vec<Dataset<'a>>,
    x_axis: Axis<'a>,
    y_axis: Axis<'a>,
    chart: Chart<'a>
}

impl<'a> GraphWidget<'a> {
    pub fn set_x_axis(&mut self, x_label: &'a str) {
        self.x_axis = Axis::default()
            .title(x_label.red())
            .style(Style::default().white())
            .bounds([0.0, 10.0])
            .labels(vec!["0.0".into(), "5.0".into(), "10.0".into()]);
    }

    pub fn set_y_axis(&mut self, y_label: &'a str) {
        self.y_axis = Axis::default()
            .title(y_label.red())
            .style(Style::default().white())
            .bounds([0.0, 10.0])
            .labels(vec!["0.0".into(), "5.0".into(), "10.0".into()]);
    }

    pub fn set_chart(&mut self, title: &'a str) {
        self.chart = Chart::new(self.datasets.clone())
            .block(Block::default().title(title))
            .x_axis(self.x_axis.clone())
            .y_axis(self.y_axis.clone());
    }
}