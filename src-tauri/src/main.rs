// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;
use damselfly3::damselfly::viewer::damselfly_viewer::DamselflyViewer;

struct AppState {
    viewer: Mutex<Option<DamselflyViewer>>,
}

fn main() {
    std::env::set_var("__NV_PRIME_RENDER_OFFLOAD", "1");
    tauri::Builder::default()
        .manage(AppState {
            viewer: None.into(),
        })
        .invoke_handler(tauri::generate_handler![
            initialise_viewer,
            get_graph,
            choose_files,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn get_viewer<'a>(state: &'a tauri::State<'_, AppState>) -> Result<&'a DamselflyViewer, String> {
    let viewer_lock = state.viewer.lock().map_err(|_| "Failed to lock viewer state".to_string())?;
    match &*viewer_lock {
        Some(viewer) => Ok(viewer),
        None => Err("Viewer is not initialised".to_string()),
    }
}

#[tauri::command]
fn get_viewer_graph(state: tauri::State<AppState>) -> Result<Vec<[f64; 2]>, String> {
    let viewer = get_viewer(&state)?;
    Ok(viewer.get_usage_graph())
}

#[tauri::command]
async fn choose_files() -> Result<String, String> {
    use tauri::api::dialog::blocking::FileDialogBuilder;
    let file = String::from(FileDialogBuilder::new().pick_file().unwrap().to_str().unwrap());
    Ok(file)
}

#[tauri::command(rename_all = "snake_case")]
fn initialise_viewer(state: tauri::State<AppState>, log_path: String, binary_path: String) {
    eprintln!("log_path: {log_path} binary: {binary_path}");
    let viewer = DamselflyViewer::new(&log_path, &binary_path);
    state.viewer.lock().unwrap().replace(viewer);
}

#[tauri::command]
fn get_graph(state: tauri::State<AppState>) -> Vec<[f64; 2]> {
    if let Some(viewer) = state.viewer.lock().unwrap().as_ref() {
        return viewer.get_usage_graph();
    }
    Vec::new()
}