use std::{error};
use eframe::Frame;
use egui::Context;
use egui_plot::{Line, Plot, PlotPoints};
use owo_colors::OwoColorize;
use crate::app::Mode::DEFAULT;
use crate::damselfly::consts::{DEFAULT_ROW_LENGTH};
use crate::damselfly::controller::DamselflyController;
use crate::damselfly::memory_parsers::MemorySysTraceParser;

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

pub enum Mode {
    DEFAULT,
    STACKTRACE,
}

/// Application.
pub struct App {
    /// Is the application running?
    pub running: bool,
    /// Damselfly
    pub damselfly_controller: DamselflyController,

    pub map_grid: Vec<Vec<usize>>,
    pub graph_scale: f64,

    pub row_length: usize,
    // Actual mapspan (e.g. becomes 100 - 200 after shifting right once)
    pub up_left_width: u16,
    pub up_right_width: u16,
    pub up_middle_width: u16,
    pub up_height: u16,
    pub down_height: u16,

    pub mode: Mode,

    pub value: f64,
    pub label: String
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new(cc: &eframe::CreationContext<'_>, trace_path: String, binary_path: String) -> Self {
        let mut mst_parser = MemorySysTraceParser::new();
        println!("Reading log file into memory: {}", trace_path.cyan());
        let log = std::fs::read_to_string(trace_path).unwrap();
        println!("Parsing instructions");
        let instructions = mst_parser.parse_log(log, binary_path);
        println!("Initialising DamselflyViewer");
        let mut damselfly_controller = DamselflyController::new();
        println!("Populating memory logs");
        damselfly_controller.viewer.load_instructions(instructions);
        App {
            running: true,
            damselfly_controller,
            map_grid: Vec::new(),
            graph_scale: 1.0,
            row_length: DEFAULT_ROW_LENGTH,
            up_left_width: 30,
            up_middle_width: 60,
            up_right_width: 30,
            up_height: 70,
            down_height: 30,
            mode: DEFAULT,
            value: 2.7,
            label: "Hello World!".to_owned(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        self.draw_top_bottom_panel(ctx);
        self.draw_central_panel(ctx);
    }
}

impl App {
    fn draw_top_bottom_panel(&mut self, ctx: &Context) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_dark_light_mode_buttons(ui);
            });
        });
    }

    fn draw_central_panel(&mut self, ctx: &Context) {
        self.draw_graph(ctx);
        /*
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Damselfly");

            ui.horizontal(|ui| {
                ui.label("Write something: ");
                ui.text_edit_singleline(&mut self.label);
            });

            ui.add(egui::Slider::new(&mut self.value, 0.0..=10.0).text("value"));
            if ui.button("Increment").clicked() {
                self.value += 1.0;
            }

            ui.separator();

            ui.add(egui::github_link_file!(
                "https://github.com/emilk/eframe_template/blob/master/",
                "Source code."
            ));

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
            });
        });
        */
    }

    fn draw_graph(&mut self, ctx: &Context) {
        let graph_data = PlotPoints::from(self.damselfly_controller.get_full_memory_usage_graph());
        let line = Line::new(graph_data);
        egui::CentralPanel::default().show(ctx, |ui| {
            let plot_response = Plot::new("plot").view_aspect(2.0).show(ui, |plot_ui| plot_ui.line(line));
        });
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to("eframe", "https://github.com/emilk/egui/tree/master/crates/eframe");
        ui.label(".");
    });
}