#![allow(non_snake_case)]

use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use std::thread::Scope;
use dioxus::prelude::*;
use damselfly4::damselfly::viewer::damselfly_viewer::DamselflyViewer;
use damselfly4::ui::file_load::FileLoad;
use damselfly4::ui::graph::Graph;
use dioxus_charts::BarChart;

fn main() {
    std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
    launch(App);
}

fn App(cx: Scope) -> Element {
    cx.render(rsx! {
        BarChart {
            padding_top: 30,
            padding_left: 70,
            padding_right: 50,
            padding_bottom: 30,
            bar_width: "10%",
            horizontal_bars: true,
            label_interpolation: |v| format!("{v}%"),
            series: vec![
                vec![63.0, 14.4, 8.0, 5.1, 1.8],
            ],
            labels: vec!["Chrome".into(), "Safari".into(), "IE/Edge".into(), "Firefox".into(), "Opera".into()]
        }
    })
    /*
    use_context_provider(|| Arc::new(Mutex::new(DamselflyViewer::default())));
    // global rerender
    use_context_provider(|| Signal::new(false));
    rsx! {
        FileLoad { }
    }
    
     */
}
