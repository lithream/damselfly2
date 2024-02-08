use std::{error};
use std::cell::RefCell;
use std::rc::Rc;
use eframe::emath::Rect;
use eframe::Frame;
use egui::{Button, Context};
use egui::style::Spacing;
use egui_plot::{Line, Plot, PlotPoint, PlotPoints};
use owo_colors::OwoColorize;
use crate::app::Mode::DEFAULT;
use crate::consts::DEFAULT_CELL_WIDTH;
use crate::damselfly::consts::{DEFAULT_ROW_LENGTH};
use crate::damselfly::controller::DamselflyController;
use crate::damselfly::map_manipulator;
use crate::damselfly::memory_parsers::MemorySysTraceParser;
use crate::damselfly::memory_structs::{MemoryStatus, MemoryUpdate, NoHashMap};

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
    pub label: String,

    pub graph_highlight: usize,
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
            graph_highlight: Default::default(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        self.draw_top_bottom_panel(ctx);
        self.draw_central_panel(ctx);
    }
}

enum GraphResponse {
    Hover(f64, f64),
    Click(f64, f64),
    None
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

    fn validate_x_coordinate(&self, x: f64) -> Result<usize, ()> {
        let int_x = x.round() as usize;
        let instructions = self.damselfly_controller.viewer.get_total_operations();
        if int_x < instructions {
            return Ok(int_x);
        }
        Err(())
    }

    fn draw_central_panel(&mut self, ctx: &Context) {
        egui::CentralPanel::default().show(ctx, |mut ui| {
            ui.vertical(|ui| {
                let current_map = self.damselfly_controller.get_current_map_state();
                let map = current_map.0;
                let latest_operation = current_map.1.cloned();
                match self.draw_graph(ui) {
                    GraphResponse::Hover(x, y) => {
                        if let Ok(temporary_graph_highlight) = self.validate_x_coordinate(x) {
                            self.damselfly_controller.graph_highlight = temporary_graph_highlight;
                        }
                    }
                    GraphResponse::Click(x, y) => {
                        if let Ok(persistent_graph_highlight) = self.validate_x_coordinate(x) {
                            self.graph_highlight = persistent_graph_highlight;
                            self.damselfly_controller.graph_highlight = persistent_graph_highlight;
                        }
                    }
                    GraphResponse::None => {
                        self.damselfly_controller.graph_highlight = self.graph_highlight;
                    }
                }
                ui.separator();
                self.draw_map(map, latest_operation, ui);
            });
        });
    }

    fn draw_graph(&mut self, ui: &mut egui::Ui) -> GraphResponse {
        let graph_data = PlotPoints::from(self.damselfly_controller.get_full_memory_usage_graph());
        let line = Line::new(graph_data);
        let hovered_point: Rc<RefCell<(f64, f64)>> = Default::default();
        let hovered_point_ref_clone: Rc<RefCell<(f64, f64)>> = Rc::clone(&hovered_point);

        let get_hovered_point_coords = move |name: &str, point: &PlotPoint| {
            *hovered_point_ref_clone.borrow_mut() = (point.x, point.y);
            format!("{:?} {:?}", name, point)
        };

        let mut graph_response = GraphResponse::None;
        let mut response = Plot::new("plot")
            .label_formatter(get_hovered_point_coords)
            .view_aspect(2.0)
            .show(ui, |plot_ui| plot_ui.line(line));
        let point = *hovered_point.borrow_mut();
        if response.response.clicked() {
            graph_response = GraphResponse::Click(point.0, point.1);
        } else if response.response.hovered() {
            graph_response = GraphResponse::Hover(point.0, point.1);
        }
        graph_response
    }

    fn draw_map(&mut self, current_map: NoHashMap<usize, MemoryStatus>, latest_operation: Option<MemoryUpdate>, ui: &mut egui::Ui) {
        let cells_per_row = 16;
        egui::Grid::new("memory_map_grid")
            .min_col_width(0.0)
            .min_row_height(0.0)
            .spacing(egui::vec2(0.0, 0.0))
            .show(ui, |ui| {
                for (cell_counter, (address, status)) in current_map.into_iter().enumerate() {
                    if cell_counter % cells_per_row == 0 {
                        ui.end_row();
                    }
                    match status {
                        MemoryStatus::Allocated(_, _, _) => {
                            if ui.add(Button::new(format!("X")).fill(egui::Color32::RED).small()).clicked() {
                                eprintln!("0x{:x}", map_manipulator::logical_to_absolute(address));
                            };
                        },
                        MemoryStatus::PartiallyAllocated(_, _) => {
                            if ui.add(Button::new(format!("=")).fill(egui::Color32::YELLOW).small()).clicked() {
                                eprintln!("0x{:x}", map_manipulator::logical_to_absolute(address));
                            };
                        }
                        MemoryStatus::Free(_) => {
                            if ui.add(Button::new(format!("0")).fill(egui::Color32::WHITE).small()).clicked() {
                                eprintln!("0x{:x}", map_manipulator::logical_to_absolute(address));
                            };
                        }
                    }
                }
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