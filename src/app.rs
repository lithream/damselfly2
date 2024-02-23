use std::{error};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;
use eframe::Frame;
use egui::{Align, Button, Color32, Context, Layout, Rect, ScrollArea, Ui, vec2, Vec2};
use egui::accesskit::DefaultActionVerb::Click;
use egui::panel::Side;
use egui_plot::{Line, Plot, PlotPoint, PlotPoints};
use egui_extras::{Column, TableBuilder};
use crate::config::app_default_config::{AppDefaultState, LowerPanelMode, MapMode};
use crate::config::app_memory_map_config::AppMemoryMapState;
use crate::consts::DEFAULT_CELL_WIDTH;
use crate::damselfly::consts::{DEFAULT_BLOCK_SIZE, DEFAULT_MEMORY_SIZE, MAX_BLOCK_SIZE};
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
        let lowest_address = viewer.get_lowest_address();
        let highest_address = viewer.get_highest_address();
        App {
            viewer,
            mode: Mode::DEFAULT,
            default_state: AppDefaultState::new(DEFAULT_BLOCK_SIZE, 4096, lowest_address, highest_address, None),
            memory_map_state: AppMemoryMapState::new(DEFAULT_BLOCK_SIZE, 4096, None),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _: &mut Frame) {
        match self.mode {
            Mode::DEFAULT => {
                self.draw_top_bottom_panel_default(ctx);
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
        let current_block = self.default_state.current_block.clone().unwrap();
        match current_block {
            MemoryUpdateType::Allocation(allocation) =>
                ui.label(allocation.get_callstack().to_string()),
            MemoryUpdateType::Free(free) =>
                ui.label(free.get_callstack().to_string()),
        };
    }

    fn draw_operation_history(&self, ui: &mut egui::Ui) {
        let label_operation = |prepend_str, operation: &MemoryUpdateType| {
            let color = match operation {
                MemoryUpdateType::Allocation(_) => Color32::RED,
                MemoryUpdateType::Free(_) => Color32::GREEN,
            };
            egui::RichText::new(format!("{prepend_str} {}", operation)).color(color)
        };

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
        ui.add(egui::Slider::new(&mut self.default_state.block_size, 1..=MAX_BLOCK_SIZE)
            .logarithmic(true)
            .drag_value_speed(0.1)
            .smart_aim(false)
            .text("BLOCK SIZE"));
        ui.add(egui::Slider::new(&mut self.default_state.map_span, 1..=DEFAULT_MEMORY_SIZE)
            .logarithmic(true)
            .drag_value_speed(0.1)
            .smart_aim(false)
            .text("MAP SPAN"));
        if ui.button("TOGGLE SNAP").clicked() {
            match self.default_state.map_mode {
                MapMode::SNAP => self.default_state.map_mode = MapMode::FULL,
                MapMode::FULL => self.default_state.map_mode = MapMode::SNAP,
            }
        }
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
                    match self.draw_graph(ui) {
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
                        GraphResponse::None => self.viewer.clear_graph_current_highlight(),
                    }
                    self.viewer.set_map_block_size(self.default_state.block_size);
                    self.default_state.current_block = Some(self.viewer.get_current_operation());
                    ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
                        self.draw_lower_panel(ui);
                    });
                });
                self.viewer.set_map_span(self.default_state.map_span);
                self.draw_map(&mut columns[1]);
            });
        });
    }

    fn draw_lower_panel(&mut self, ui: &mut Ui) {
        ui.columns(2, |columns| {
            self.draw_operation_history(&mut columns[0]);
            columns[1].with_layout(Layout::top_down(Align::LEFT), |tabbed_panel| {
                tabbed_panel.with_layout(Layout::left_to_right(Align::LEFT), |tabs| {
                    if tabs.button("SETTINGS").clicked() { self.default_state.lower_panel_mode = LowerPanelMode::SETTINGS; }
                    if tabs.button("CALLSTACK").clicked() { self.default_state.lower_panel_mode = LowerPanelMode::CALLSTACK; }
                    if tabs.button("STATISTICS").clicked() { self.default_state.lower_panel_mode = LowerPanelMode::STATISTICS; }
                });
                match self.default_state.lower_panel_mode {
                    LowerPanelMode::SETTINGS => self.draw_settings(tabbed_panel),
                    LowerPanelMode::CALLSTACK => self.draw_callstack(tabbed_panel),
                    LowerPanelMode::STATISTICS => self.draw_statistics(tabbed_panel),
                }
            })
        });
    }

    fn draw_statistics(&self, ui: &mut Ui) {
        let (largest_free_block, free_blocks) = self.viewer.get_free_blocks_stats();
        ui.label(format!("Largest free block: {largest_free_block}"));
        ui.label(format!("Free blocks: {free_blocks}"));
    }


    fn draw_central_panel_memorymap(&mut self, ctx: &Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.draw_settings(ui);
        });
    }

    fn draw_map(&mut self, ui: &mut egui::Ui) {
        if let Some(cached_map) = self.get_cached_map() {
            for (rect, color) in cached_map {
                ui.painter().rect_filled(*rect, 0.0, *color);
            }
        }

        self.viewer.set_map_span(self.default_state.map_span);
        // Otherwise, cache a new map
        let mut new_cached_map = (self.viewer.get_graph_highlight(), Vec::new());
        let blocks = match self.default_state.map_mode {
            MapMode::SNAP => self.viewer.get_map(),
            MapMode::FULL => self.viewer.get_map_full(),
        };
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