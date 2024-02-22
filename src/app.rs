use std::{error};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;
use eframe::Frame;
use egui::{Align, Button, Color32, Context, Layout, Rect, ScrollArea, Ui, vec2, Vec2};
use egui::panel::Side;
use egui_plot::{Line, Plot, PlotPoint, PlotPoints};
use egui_extras::{Column, TableBuilder};
use crate::config::app_default_config::AppDefaultState;
use crate::config::app_memory_map_config::AppMemoryMapState;
use crate::consts::DEFAULT_CELL_WIDTH;
use crate::damselfly::consts::{DEFAULT_BLOCK_SIZE, MAX_BLOCK_SIZE};
use crate::damselfly::memory::memory_status::MemoryStatus;
use crate::damselfly::memory::memory_update::{MemoryUpdate, MemoryUpdateType};
use crate::damselfly::viewer::damselfly_viewer::DamselflyViewer;

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

pub enum Mode {
    DEFAULT,
    MEMORYMAP,
}

/// Application.
pub struct App {
    viewer: DamselflyViewer,
    mode: Mode,
    default_state: AppDefaultState,
    memory_map_state: AppMemoryMapState,
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new(_: &eframe::CreationContext<'_>, log_path: String, binary_path: String) -> Self {
        let viewer = DamselflyViewer::new(log_path.as_str(), binary_path.as_str());
        App {
            viewer,
            mode: Mode::DEFAULT,
            default_state: AppDefaultState::new(DEFAULT_BLOCK_SIZE, 4096, None),
            memory_map_state: AppMemoryMapState::new(DEFAULT_BLOCK_SIZE, 4096, None),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _: &mut Frame) {
        match self.mode {
            Mode::DEFAULT => {
                self.draw_top_bottom_panel_default(ctx);
//                self.draw_side_panel_default(ctx);
                self.draw_central_panel_default(ctx);
            }
            Mode::MEMORYMAP => {
                self.draw_central_panel_memorymap(ctx);
                self.draw_top_bottom_panel_default(ctx);
                self.draw_central_panel_memorymap(ctx);
            }
        }
    }
}

enum GraphResponse {
    Hover(f64, f64),
    Click(f64, f64),
    None
}

impl App {
    fn draw_top_bottom_panel_default(&mut self, ctx: &Context) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            self.draw_title_bar(ctx, ui);
        });
    }

    fn draw_callstack(&self, ui: &mut egui::Ui) {
        if self.default_state.current_block.is_none() { return };
        egui::ScrollArea::vertical().show(ui, |ui| {
            let current_block = self.default_state.current_block.clone().unwrap();
            match current_block {
                MemoryUpdateType::Allocation(allocation) =>
                    ui.label(allocation.get_callstack().to_string()),
                MemoryUpdateType::Free(free) =>
                    ui.label(free.get_callstack().to_string()),
            };
        });
    }

    fn draw_operation_history(&self, ui: &mut egui::Ui) {
        let label_operation = |prepend_str, operation: &MemoryUpdateType| {
            let color = match operation {
                MemoryUpdateType::Allocation(_) => Color32::RED,
                MemoryUpdateType::Free(_) => Color32::GREEN,
            };
            egui::RichText::new(format!("{prepend_str} {}", operation)).color(color)
        };

        ScrollArea::vertical().show(ui, |ui| {
            let operation_history = self.viewer.get_operation_history();
            TableBuilder::new(ui)
                .column(Column::remainder())
                .body(|mut body| {
                    if let Some(first_operation) = operation_history.first() {
                        body.row(10.0, |mut row| {
                            row.col(|ui| {
                                ui.label(label_operation("->", first_operation));
                            });
                        });
                    }
                    for operation in operation_history.iter().skip(1) {
                        body.row(10.0, |mut row| {
                            row.col(|ui| {
                                ui.label(label_operation("", operation));
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

    fn draw_settings(&mut self, ui: &mut egui::Ui) {
        ui.add(egui::Slider::new(&mut self.default_state.block_size, 16..=MAX_BLOCK_SIZE)
            .logarithmic(true)
            .drag_value_speed(0.1)
            .text("BLOCK SIZE"));
    }

    fn draw_title_bar(&mut self, ctx: &Context, ui: &mut egui::Ui) {
        egui::menu::bar(ui, |ui| {
            let is_web = cfg!(target_arch = "wasm32");
            if !is_web {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.add_space(16.0);
                ui.menu_button("View", |ui| {
                    if ui.button("Default").clicked() {
                        self.mode = Mode::DEFAULT;
                    }
                    if ui.button("Memory map").clicked() {
                        self.mode = Mode::MEMORYMAP;
                    }
                });
            }

            egui::widgets::global_dark_light_mode_buttons(ui);
        });
    }

    fn draw_central_panel_default(&mut self, ctx: &Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.columns(2, |columns| {
                columns[0].with_layout(Layout::top_down(Align::LEFT), |ui| {
                    self.draw_graph(ui);
                    ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
                        self.draw_lower_panel(ui);
                        self.draw_settings(ui);
                        self.draw_operation_history(ui);
                    });
                });
                self.draw_debug_map(&mut columns[1]);
            });
        });
            // left half

            /*
            ui.columns(2, |columns| {
                columns[0].label("USAGE");
                /*
                TableBuilder::new(columns[0])
                    .column(Column::remainder())
                    .body(|mut body| {
                        body.row
                    })
                 */

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

                columns[0].separator();
                self.draw_callstack(&mut columns[0]);
                self.viewer.set_map_block_size(self.default_state.block_size);
                self.default_state.current_block = Some(self.viewer.get_current_operation());

                columns[1].label("MEMORY");
                self.draw_full_map(&mut columns[1]);
            })
        });
             */
    }

    fn draw_central_panel_memorymap(&mut self, ctx: &Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.draw_settings(ui);
        });
    }

    /*
    fn cache_maps(&mut self) {
        let mut cached_maps = Vec::new();
        for timestamp in 0..self.viewer.get_total_operations() {
            cached_maps.push(self.viewer.get_map_full_at(timestamp));
        }
    }
     */

    fn draw_full_map(&mut self, ui: &mut egui::Ui) {
        ScrollArea::vertical().show(
            ui,
            |ui| {
                if let Some(cached_map) = self.get_cached_map() {
                    let start = Instant::now();
                    for (rect, color) in cached_map {
                        ui.painter().rect_filled(*rect, 0.0, *color);
                    }
                    println!("Time to draw cached map: {:?}", start.elapsed());
                }

                // Otherwise, cache a new map
                let start_time = Instant::now();
                let mut new_cached_map = (self.viewer.get_graph_highlight(), Vec::new());
                let blocks = self.viewer.get_map_full();
                let start = ui.min_rect().min;
                let end = ui.min_rect().max;

                let mut cur_x = 0.0;
                let mut cur_y = 0.0;

                let mut consecutive_identical_blocks = 0;

                for (index, block) in blocks.iter().enumerate() {
                    if let Some(prev_block) = blocks.get(index.saturating_sub(1)) {
                        if prev_block == block {
                            consecutive_identical_blocks += 1;
                        } else {
                            consecutive_identical_blocks = 0;
                        }
                    }

                    if consecutive_identical_blocks > 512 {
                        continue;
                    }

                    if cur_x >= end.x {
                        cur_x = 0.0;
                        cur_y += DEFAULT_CELL_WIDTH;
                    } else {
                        cur_x += DEFAULT_CELL_WIDTH;
                    }

                    let rect = Rect::from_min_size(
                        start + vec2(cur_x, cur_y),
                        vec2(DEFAULT_CELL_WIDTH, DEFAULT_CELL_WIDTH),
                    );

                    let color = match block {
                        MemoryStatus::Allocated(..) => Color32::RED,
                        MemoryStatus::PartiallyAllocated(..) => Color32::YELLOW,
                        MemoryStatus::Free(..) => Color32::GREEN,
                        MemoryStatus::Unused => Color32::WHITE,
                    };

                    new_cached_map.1.push((rect, color));
                }
                self.memory_map_state.cache_map(new_cached_map);
                println!("Time to generate cached map: {:?}", start_time.elapsed());
            }
        );
    }

    fn get_cached_map(&self) -> Option<&Vec<(Rect, Color32)>> {
        // Check if cached map exists
        if let Some((timestamp, cached_map)) = self.memory_map_state.get_cached_map() {
            // Check if cached map corresponds to current timestamp
            if *timestamp == self.viewer.get_graph_highlight() {
                return Some(cached_map);
            }
        }
        None
    }

    fn draw_graph(&mut self, ui: &mut Ui) -> GraphResponse {
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
                .spacing(vec2(0.0, 0.0))
                .show(ui, |ui| {
                    for (index, block) in blocks.iter().enumerate() {
                        if index % cells_per_row == 0 {
                            ui.end_row();
                        }

                        match block {
                            MemoryStatus::Allocated(parent_address, parent_size, callstack) => {
                                if ui.add(Button::new("X".to_string()).fill(Color32::RED).small()).clicked() {
                                    eprintln!("ALLOC 0x{:x} {}B {}", parent_address, parent_size, *callstack);
                                }
                            }
                            MemoryStatus::PartiallyAllocated(parent_address, parent_size, callstack) => {
                                if ui.add(Button::new("=".to_string()).fill(Color32::YELLOW).small()).clicked() {
                                    eprintln!("0x{:x} {}B {}", parent_address, parent_size, *callstack);
                                }
                            }
                            MemoryStatus::Free(parent_address, parent_size, callstack) => {
                                if ui.add(Button::new("0".to_string()).fill(Color32::WHITE).small()).clicked() {
                                    eprintln!("0x{:x} {}B {}", parent_address, parent_size, callstack);
                                }
                            }
                            MemoryStatus::Unused => {
                                if ui.add(Button::new("U".to_string()).fill(Color32::WHITE).small()).clicked() {
                                    eprintln!("UNUSED");
                                }
                            }
                        }
                    }
                });
        });
    }

    fn draw_side_panel_default(&mut self, ctx: &Context) {
        egui::SidePanel::new(Side::Left, "Right panel").show(ctx, |ui| {
            eprintln!("drawing map");
            self.draw_map_controls(ui);
            ui.separator();
            eprintln!("drawing operation");
            self.draw_operation_history(ui);
            ui.separator();
            eprintln!("calculating free");
            let (largest_free_block, free_blocks) = self.viewer.get_free_blocks_stats();
            ui.label(format!("{}", largest_free_block));
            ui.separator();
            ui.label(format!("{}", free_blocks));
        });
    }

    fn draw_debug_map(&mut self, ui: &mut egui::Ui) {
        let start = ui.min_rect().min;
        let end = ui.min_rect().max;

        let start_rect = Rect::from_min_size(start, vec2(DEFAULT_CELL_WIDTH, DEFAULT_CELL_WIDTH));
        let end_rect = Rect::from_min_size(end, vec2(DEFAULT_CELL_WIDTH, DEFAULT_CELL_WIDTH));
        ui.painter().rect_filled(start_rect, 0.0, Color32::RED);
        ui.painter().rect_filled(end_rect, 0.0, Color32::RED);
    }

    fn draw_map_controls(&mut self, ui: &mut egui::Ui) {
        ui.add(egui::Slider::new(&mut self.default_state.block_size, 1..=MAX_BLOCK_SIZE)
            .logarithmic(true)
            .smart_aim(false)
            .drag_value_speed(0.1)
            .text("BLOCK SIZE"));
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