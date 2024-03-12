use std::sync::{Arc, Mutex};
use std::thread::Scope;
use dioxus::prelude::*;
use crate::ui::utility::get_viewer;
use dioxus_charts::LineChart;

pub fn Graph(cx: Scope) -> Element {
    consume_context::<Signal<bool>>().read();
    let graph_data = get_graph_data();
    rsx! {
    }
}

fn get_graph_data() -> Vec<[f64; 2]> {
    let damselfly_viewer = get_viewer().clone();
    let lock = damselfly_viewer.lock().expect("[Ui::get_graph_data]: Failed to lock viewer");
    lock.get_usage_graph()
}