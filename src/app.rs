use std::{error};
use std::cell::RefCell;
use std::rc::Rc;
use eframe::Frame;
use egui::{Button, Context};
use egui::panel::Side;
use egui_plot::{Line, Plot, PlotPoint, PlotPoints};
use egui_extras::{Column, TableBuilder};
use crate::consts::DEFAULT_CELL_WIDTH;
use crate::damselfly::consts::{DEFAULT_BLOCK_SIZE, DEFAULT_MEMORYSPAN, MAX_BLOCK_SIZE, MAX_MAP_SPAN};
use crate::damselfly::memory::memory_status::MemoryStatus;
use crate::damselfly::memory::memory_update::{MemoryUpdate, MemoryUpdateType};
use crate::damselfly::viewer::damselfly_viewer::DamselflyViewer;

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

pub enum Mode {
    DEFAULT,
    STACKTRACE,
}

/// Application.
pub struct App {
    pub viewer: DamselflyViewer,
    pub block_size: usize,
    pub map_span: usize,
    pub current_block: Option<MemoryUpdateType>,
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new(_: &eframe::CreationContext<'_>, log_path: String, binary_path: String) -> Self {
        let viewer = DamselflyViewer::new(log_path.as_str(), binary_path.as_str());
        App {
            viewer,
            block_size: DEFAULT_BLOCK_SIZE,
            map_span: DEFAULT_MEMORYSPAN,
            current_block: None,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _: &mut Frame) {
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
            self.draw_title_bar(ctx, ui);
        });

        egui::TopBottomPanel::bottom("bottom_panel")
            .default_height(50.0)
            .resizable(true)
            .show(ctx, |ui| {
            ui.columns(2, |columns| {
                columns[0].label("CALLSTACK");
                columns[1].label("HISTORY");
                self.draw_callstack(&mut columns[0]);
                self.draw_operation_history(&mut columns[1]);
            })
        });
    }

    fn draw_callstack(&self, ui: &mut egui::Ui) {
        if self.current_block.is_none() { return };
        egui::ScrollArea::vertical().show(ui, |ui| {
            let current_block = self.current_block.clone().unwrap();
            match current_block {
                MemoryUpdateType::Allocation(allocation) =>
                    ui.label(allocation.get_callstack().to_string()),
                MemoryUpdateType::Free(free) =>
                    ui.label(free.get_callstack().to_string()),
            };
        });
    }

    fn draw_operation_history(&self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            let operation_history = self.viewer.get_operation_history();
            TableBuilder::new(ui)
                .column(Column::remainder())
                .body(|mut body| {
                    for operation in operation_history {
                        body.row(30.0, |mut row| {
                            row.col(|ui| {
                                ui.label(operation.to_string());
                            });
                        });
                    }
                });
        });
    }

    fn validate_x_coordinate(&self, x: f64) -> Result<usize, ()> {
        let int_x = x.round() as usize;
        let instructions = self.viewer.get_total_operations();
        if int_x < instructions {
            return Ok(int_x);
        }
        Err(())
    }

    fn draw_title_bar(&self, ctx: &Context, ui: &mut egui::Ui) {
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
    }

    fn draw_central_panel(&mut self, ctx: &Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.columns(2, |columns| {
                columns[0].label("USAGE");
                columns[1].label("MEMORY");
                match self.draw_graph(&mut columns[0]) {
                    GraphResponse::Hover(x, _) => {
                        if let Ok(temporary_graph_highlight) = self.validate_x_coordinate(x) {
                            self.viewer.set_graph_current_highlight(temporary_graph_highlight);
                        }
                    }
                    GraphResponse::Click(x, _) => {
                        if let Ok(persistent_graph_highlight) = self.validate_x_coordinate(x) {
                            self.viewer.set_graph_saved_highlight(persistent_graph_highlight);
                        }
                    }
                    GraphResponse::None => {
                        self.viewer.clear_graph_current_highlight();
                    }
                }
                let pane_width = columns[1].available_width();
                self.viewer.set_map_block_size(self.block_size);
                self.viewer.set_map_span(self.map_span);
                self.current_block = Some(self.viewer.get_current_operation());
                let map = self.viewer.get_map();
                self.draw_map(map, &mut columns[1], pane_width);
            })
        });
    }

    fn draw_graph(&mut self, ui: &mut egui::Ui) -> GraphResponse {
        let hovered_point: Rc<RefCell<(f64, f64)>> = Default::default();
        let hovered_point_ref_clone: Rc<RefCell<(f64, f64)>> = Rc::clone(&hovered_point);

        let get_hovered_point_coords = move |name: &str, point: &PlotPoint| {
            *hovered_point_ref_clone.borrow_mut() = (point.x, point.y);
            format!("{:?} {:?}", name, point)
        };

        let usage_data = PlotPoints::from(self.viewer.get_usage_graph());
        let usage_line = Line::new(usage_data);
        let distinct_blocks_data = PlotPoints::from(self.viewer.get_distinct_blocks_graph());
        let distinct_blocks_line = Line::new(distinct_blocks_data);

        let mut graph_response = GraphResponse::None;
        let response = Plot::new("plot")
            .label_formatter(get_hovered_point_coords)
            .view_aspect(2.0)
            .show(ui, |plot_ui| {
                plot_ui.line(usage_line);
                plot_ui.line(distinct_blocks_line);
            });

        let point = *hovered_point.borrow_mut();
        if response.response.clicked() {
            graph_response = GraphResponse::Click(point.0, point.1);
        } else if response.response.hovered() {
            graph_response = GraphResponse::Hover(point.0, point.1);
        }
        graph_response
    }

    fn draw_map(&mut self,
                blocks: Vec<MemoryStatus>,
                ui: &mut egui::Ui,
                pane_width: f32
    ) {
        let cells_per_row = pane_width as usize / DEFAULT_CELL_WIDTH as usize;
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("memory_map_grid")
                .min_col_width(0.0)
                .min_row_height(0.0)
                .spacing(egui::vec2(0.0, 0.0))
                .show(ui, |ui| {
                    for (index, block) in blocks.iter().enumerate() {
                        if index % cells_per_row == 0 {
                            ui.end_row();
                        }

                        match block {
                            MemoryStatus::Allocated(parent_address, parent_size, callstack) => {
                                if ui.add(Button::new("X".to_string()).fill(egui::Color32::RED).small()).clicked() {
                                    eprintln!("ALLOC 0x{:x} {}B {}", parent_address, parent_size, *callstack);
                                }
                            }
                            MemoryStatus::PartiallyAllocated(parent_address, parent_size, callstack) => {
                                if ui.add(Button::new("=".to_string()).fill(egui::Color32::YELLOW).small()).clicked() {
                                    eprintln!("0x{:x} {}B {}", parent_address, parent_size, *callstack);
                                }
                            }
                            MemoryStatus::Free(parent_address, parent_size, callstack) => {
                                if ui.add(Button::new("0".to_string()).fill(egui::Color32::WHITE).small()).clicked() {
                                    eprintln!("0x{:x} {}B {}", parent_address, parent_size, callstack);
                                }
                            }
                            MemoryStatus::Unused => {
                                if ui.add(Button::new("U".to_string()).fill(egui::Color32::WHITE).small()).clicked() {
                                    eprintln!("UNUSED");
                                }
                            }
                        }
                    }
                });
        });
    }

    fn draw_side_panel(&mut self, ctx: &Context) {
        egui::SidePanel::new(Side::Right, "Right panel").show(ctx, |ui| {
            self.draw_map_controls(ui);
        });
    }

    fn draw_map_controls(&mut self, ui: &mut egui::Ui) {
        ui.add(egui::Slider::new(&mut self.block_size, 1..=MAX_BLOCK_SIZE)
            .logarithmic(true)
            .smart_aim(false)
            .drag_value_speed(0.1)
            .text("BLOCK SIZE"));
        ui.add(egui::Slider::new(&mut self.map_span, 1..=MAX_MAP_SPAN)
            .logarithmic(true)
            .smart_aim(false)
            .drag_value_speed(0.1)
            .text("CANVAS SPAN"));
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