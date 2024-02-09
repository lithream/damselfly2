use std::{error};
use std::cell::RefCell;
use std::rc::Rc;
use eframe::Frame;
use egui::{Button, Context, Slider};
use egui_plot::{Line, Plot, PlotPoint, PlotPoints};
use owo_colors::OwoColorize;
use crate::app::Mode::DEFAULT;
use crate::consts::DEFAULT_CELL_WIDTH;
use crate::damselfly::consts::{DEFAULT_ROW_LENGTH};
use crate::damselfly::controller::DamselflyController;
use crate::damselfly::{consts, map_manipulator};
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
    pub damselfly_controller: DamselflyController,
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
        damselfly_controller.viewer.load_instructions(instructions, consts::_BLOCK_SIZE);
        App {
            damselfly_controller,
            graph_highlight: Default::default(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        self.draw_top_bottom_panel(ctx);
        self.draw_side_panel(ctx);
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
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.columns(2, |columns| {
                columns[0].label("USAGE");
                columns[1].label("MEMORY");
                match self.draw_graph(&mut columns[0]) {
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
                let current_map = self.damselfly_controller.get_current_map_state();
                let map = current_map.0;
                let latest_operation = current_map.1.cloned();
                let pane_width = columns[1].available_width();
                self.draw_map(map, latest_operation, &mut columns[1], pane_width);
            })
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

    fn draw_map(&mut self, current_map: NoHashMap<usize, MemoryStatus>, latest_operation: Option<MemoryUpdate>, ui: &mut egui::Ui, pane_width: f32) {
        let cells_per_row = pane_width as usize / DEFAULT_CELL_WIDTH as usize;
        let span = self.damselfly_controller.memory_span;
        let block_size = self.damselfly_controller.block_size;
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("memory_map_grid")
                .min_col_width(0.0)
                .min_row_height(0.0)
                .spacing(egui::vec2(0.0, 0.0))
                .show(ui, |ui| {
                    for address in span.0..=span.1 {
                        if (address - span.0) % cells_per_row == 0 {
                            ui.end_row();
                        }
                        match current_map.get(&address) {
                            Some(status) => {
                                match status {
                                    MemoryStatus::Allocated(_, _, _) => {
                                        if ui.add(Button::new("X".to_string()).fill(egui::Color32::RED).small()).clicked() {
                                            eprintln!("0x{:x}", map_manipulator::logical_to_absolute(address, block_size));
                                        };
                                    },
                                    MemoryStatus::PartiallyAllocated(_, _) => {
                                        if ui.add(Button::new("=".to_string()).fill(egui::Color32::YELLOW).small()).clicked() {
                                            eprintln!("0x{:x}", map_manipulator::logical_to_absolute(address, block_size));
                                        };
                                    }
                                    MemoryStatus::Free(_) => {
                                        if ui.add(Button::new("0".to_string()).fill(egui::Color32::WHITE).small()).clicked() {
                                            eprintln!("0x{:x}", map_manipulator::logical_to_absolute(address, block_size));
                                        };
                                    }
                                }
                            }
                            None => {
                                ui.add(Button::new("U".to_string()).fill(egui::Color32::WHITE).small());
                            }
                        }
                    }
                });
        });
    }

    fn draw_side_panel(&mut self, ctx: &Context) {
        egui::SidePanel::right("RIGHT PANEL").show(&ctx, |ui| {
            ui.add(Slider::new(&mut self.damselfly_controller.block_size, 4..=4096).text("BLOCK SIZE"));
        });
    }
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