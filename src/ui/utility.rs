use std::sync::{Arc, Mutex};
use dioxus::hooks::use_context;
use crate::damselfly::viewer::damselfly_viewer::DamselflyViewer;

pub fn get_viewer () -> Arc<Mutex<DamselflyViewer>> {
    Arc::clone(&use_context::<Arc<Mutex<DamselflyViewer>>>())
}