// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::ops::DerefMut;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use damselfly3::damselfly::memory::memory_status::MemoryStatus;
use damselfly3::damselfly::viewer::damselfly_viewer::DamselflyViewer;

struct AppState {
    viewer: Arc<Mutex<Option<DamselflyViewer>>>,
}

fn main() {
    std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
    tauri::Builder::default()
        .manage(AppState {
            viewer: Arc::new(Mutex::new(None)),
        })
        .invoke_handler(tauri::generate_handler![
            initialise_viewer,
            get_viewer_usage_graph,
            get_viewer_usage_graph_sampled,
            get_viewer_fragmentation_graph,
            get_viewer_largest_block_graph,
            get_viewer_free_blocks_graph,
            get_viewer_map_full_at_colours,
            choose_files,
            set_block_size,
            get_operation_log,
            get_callstack,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command(rename_all = "snake_case")]
fn initialise_viewer(state: tauri::State<AppState>, log_path: String, binary_path: String) {
    let viewer = DamselflyViewer::new(&log_path, &binary_path);
    state.viewer.lock().unwrap().replace(viewer);
}

#[tauri::command]
async fn choose_files() -> Result<String, String> {
    use tauri::api::dialog::blocking::FileDialogBuilder;
    let file = String::from(FileDialogBuilder::new().pick_file().unwrap().to_str().unwrap());
    Ok(file)
}

#[tauri::command]
fn get_viewer_usage_graph(state: tauri::State<AppState>) -> Result<Vec<[f64; 2]>, String> {
    let viewer_lock = state.viewer.lock().unwrap();
    if let Some(viewer) = &*viewer_lock {
        Ok(viewer.get_usage_graph())
    } else {
        Err("Viewer is not initialised".to_string())
    }
}

#[tauri::command]
fn get_viewer_usage_graph_sampled(state: tauri::State<AppState>) -> Result<Vec<[f64; 2]>, String> {
    let viewer_lock = state.viewer.lock().unwrap();
    if let Some(viewer) = &*viewer_lock {
        Ok(viewer.get_usage_graph_realtime_sampled())
    } else {
        Err("Viewer is not initialised".to_string())
    }
}

#[tauri::command]
fn get_viewer_fragmentation_graph(state: tauri::State<AppState>) -> Result<Vec<[f64; 2]>, String> {
    let viewer_lock = state.viewer.lock().unwrap();
    if let Some(viewer) = &*viewer_lock {
        Ok(viewer.get_distinct_blocks_graph())
    } else {
        Err("Viewer is not initialised".to_string())
    }
}

#[tauri::command]
fn get_viewer_largest_block_graph(state: tauri::State<AppState>) -> Result<Vec<[f64; 2]>, String> {
    let viewer_lock = state.viewer.lock().unwrap();
    if let Some(viewer) = &*viewer_lock {
        Ok(viewer.get_largest_block_graph())
    } else {
        Err("Viewer is not initialised".to_string())
    }
}

#[tauri::command]
fn get_viewer_free_blocks_graph(state: tauri::State<AppState>) -> Result<Vec<[f64; 2]>, String> {
    let viewer_lock = state.viewer.lock().unwrap();
    if let Some(viewer) = &*viewer_lock {
        Ok(viewer.get_free_blocks_graph())
    } else {
        Err("Viewer is not initialised".to_string())
    }
}

#[tauri::command]
fn get_viewer_map(state: tauri::State<AppState>) -> Result<Vec<MemoryStatus>, String> {
    let viewer_lock = state.viewer.lock().unwrap();
    if let Some(viewer) = &*viewer_lock {
        Ok(viewer.get_map_full_nosync())
    } else {
        Err("Viewer is not initialised".to_string())
    }
}

#[tauri::command]
fn get_viewer_map_full_at(state: tauri::State<AppState>, timestamp: usize) -> Result<Vec<MemoryStatus>, String> {
    eprintln!("timestamp: {timestamp}");
    let mut viewer_lock = state.viewer.lock().unwrap();
    if let Some(viewer) = viewer_lock.deref_mut() {
        Ok(viewer.get_map_full_at_nosync(timestamp))
    } else {
        Err("Viewer is not initialised".to_string())
    }
}

#[tauri::command]
fn get_viewer_map_full_at_colours(state: tauri::State<AppState>, timestamp: u64, truncate_after: u64) -> Result<(u64, Vec<(i64, u64)>), String> {
    let mut viewer_lock = state.viewer.lock().unwrap();
    if let Some(viewer) = viewer_lock.deref_mut() {
        let res = viewer.get_map_full_at_nosync_colours_truncate(timestamp, truncate_after);
        Ok(res)
    } else {
        Err("Viewer is not initialised".to_string())
    }
}

#[tauri::command]
fn set_block_size(state: tauri::State<AppState>, new_block_size: u64) -> Result<(), String> {
    let mut viewer_lock = state.viewer.lock().unwrap();
    if let Some(viewer) = viewer_lock.deref_mut() {
        viewer.set_map_block_size(new_block_size as usize);
        Ok(())
    } else {
        Err("Viewer is not initialised".to_string())
    }
}

#[tauri::command]
fn get_operation_log(state: tauri::State<AppState>) -> Result<Vec<String>, String> {
    let mut viewer_lock = state.viewer.lock().unwrap();
    if let Some(viewer) = viewer_lock.deref_mut() {
        Ok(viewer.get_operation_history()
            .iter()
            .take(7)
            .map(|update| update.to_string())
            .collect())
    } else {
        Err("Viewer is not initialised".to_string())
    }
}

#[tauri::command]
fn get_callstack(state: tauri::State<AppState>) -> Result<String, String> {
    let mut viewer_lock = state.viewer.lock().unwrap();
    if let Some(viewer) = viewer_lock.deref_mut() {
        Ok(viewer.get_current_operation().get_callstack().to_string())
    } else {
        Err("Viewer is not initialised".to_string())
    }
}
