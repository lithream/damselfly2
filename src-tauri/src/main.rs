// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{Arc, Mutex};
use damselfly3::damselfly::viewer::damselfly_viewer::DamselflyViewer;

struct AppState {
    viewer: Arc<Mutex<Option<DamselflyViewer>>>,
}

fn main() {
    std::env::set_var("__NV_PRIME_RENDER_OFFLOAD", "1");
    tauri::Builder::default()
        .manage(AppState {
            viewer: Arc::new(Mutex::new(None)),
        })
        .invoke_handler(tauri::generate_handler![
            initialise_viewer,
            get_viewer_graph,
            choose_files,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn get_viewer_graph(state: tauri::State<AppState>) -> Result<Vec<[f64; 2]>, String> {
    let viewer_lock = state.viewer.lock().unwrap();
    if let Some(viewer) = &*viewer_lock {
        Ok(viewer.get_usage_graph())
    } else {
        Err("Viewer is not initialised".to_string())
    }
}

#[tauri::command]
async fn choose_files() -> Result<String, String> {
    use tauri::api::dialog::blocking::FileDialogBuilder;
    let file = String::from(FileDialogBuilder::new().pick_file().unwrap().to_str().unwrap());
    Ok(file)
}

#[tauri::command(rename_all = "snake_case")]
fn initialise_viewer(state: tauri::State<AppState>, log_path: String, binary_path: String) {
    let viewer = DamselflyViewer::new(&log_path, &binary_path);
    (&*state.viewer).lock().unwrap().replace(viewer);
}