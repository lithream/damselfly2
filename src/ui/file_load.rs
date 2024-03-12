use std::sync::{Arc, Mutex};
use dioxus::prelude::*;
use log::Level;
use crate::damselfly::viewer::damselfly_viewer::DamselflyViewer;
use rfd::FileDialog;
use crate::ui::utility::get_viewer;

pub fn FileLoad() -> Element {
    let damselfly_viewer = get_viewer();
    let load_viewer = move |_| { load_viewer(damselfly_viewer.clone()) };

    rsx! {
        button {
            onclick: load_viewer,
            "Load"
        }
    }
}

fn pick_file() -> Result<String, String> {
    return if let Some(file) = FileDialog::new().pick_file() {
        if let Some(file_path) = file.as_path().to_str() {
            Ok(file_path.to_string())
        } else {
            Err("[Ui::FileLoad]: Unable to parse file path to String".to_string())
        }
    } else {
        Err("[Ui::FileLoad]: Unable to open log file".to_string())
    }
}

fn load_viewer(damselfly_viewer: Arc<Mutex<DamselflyViewer>>) {
    let mut global_rerender = consume_context::<Signal<bool>>();
    let log_file = pick_file();
    let binary_file = pick_file();
    if log_file.is_err() {
        log::log!(Level::Error, "[Ui::FileLoad::load_viewer]: Invalid log file");
        return;
    }
    if binary_file.is_err() {
        log::log!(Level::Error, "[Ui::FileLoad::load_viewer]: Invalid binary file");
        return;
    }
    let (log_file, binary_file) = (log_file.unwrap(), binary_file.unwrap());
    damselfly_viewer
        .lock()
        .expect("[Ui::load_viewer]: Unable to lock DamselflyViewer")
        .load(&log_file, &binary_file);
    *global_rerender.write() = true;
}
