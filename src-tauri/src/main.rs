// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use damselfly3::damselfly::memory::memory_status::MemoryStatus;
use damselfly3::damselfly::memory::memory_update::MemoryUpdateType;
use damselfly3::damselfly::viewer::damselfly_viewer::DamselflyViewer;
use std::ops::DerefMut;
use std::sync::{Arc, Mutex};

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
            get_viewer_distinct_blocks_graph,
            get_viewer_distinct_blocks_graph_sampled,
            get_viewer_largest_block_graph,
            get_viewer_largest_block_graph_sampled,
            get_viewer_free_blocks_graph,
            get_viewer_free_blocks_graph_sampled,
            get_viewer_map_full_at_colours,
            get_viewer_map_full_at_colours_realtime_sampled,
            choose_files,
            set_block_size,
            get_operation_log,
            get_callstack,
            query_block,
            query_block_realtime,
            get_pool_list,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command(rename_all = "snake_case")]
fn initialise_viewer(state: tauri::State<AppState>, log_path: String, binary_path: String, cache_size: u64) {
    let viewer = DamselflyViewer::new(&log_path, &binary_path, cache_size);
    state.viewer.lock().unwrap().replace(viewer);
}

#[tauri::command]
async fn choose_files() -> Result<String, String> {
    use tauri::api::dialog::blocking::FileDialogBuilder;
    let file = String::from(
        FileDialogBuilder::new()
            .pick_file()
            .unwrap()
            .to_str()
            .unwrap(),
    );
    Ok(file)
}

#[tauri::command]
fn get_viewer_usage_graph(state: tauri::State<AppState>, damselfly_instance: u64) -> Result<Vec<[f64; 2]>, String> {
    let mut viewer_lock = state.viewer.lock().unwrap();
    if let Some(viewer) = &mut *viewer_lock {
        Ok(viewer
            .damselflies
            .get_mut(damselfly_instance as usize)
            .expect("[tauri::command::get_viewer_usage_graph]: damselfly_instance not found: {damselfly_instance}")
            .get_usage_graph())
    } else {
        Err("Viewer is not initialised".to_string())
    }
}

#[tauri::command]
fn get_viewer_usage_graph_sampled(state: tauri::State<AppState>, damselfly_instance: u64) -> Result<Vec<[f64; 2]>, String> {
    let mut viewer_lock = state.viewer.lock().unwrap();
    if let Some(viewer) = &mut *viewer_lock {
        Ok(viewer
            .damselflies
            .get_mut(damselfly_instance as usize)
            .expect("[tauri::command::get_viewer_usage_graph_sampled]: damselfly_instance not found: {damselfly_instance}")
           .get_usage_graph_realtime_sampled())
    } else {
        Err("Viewer is not initialised".to_string())
    }
}

#[tauri::command]
fn get_viewer_distinct_blocks_graph(state: tauri::State<AppState>, damselfly_instance: u64) -> Result<Vec<[f64; 2]>, String> {
    let mut viewer_lock = state.viewer.lock().unwrap();
    if let Some(viewer) = &mut *viewer_lock {
        Ok(viewer
            .damselflies
            .get_mut(damselfly_instance as usize)
            .expect("[tauri::command::get_viewer_distinct_blocks_graph]: damselfly_instance not found: {damselfly_instance}")
            .get_distinct_blocks_graph())
    } else {
        Err("Viewer is not initialised".to_string())
    }
}

#[tauri::command]
fn get_viewer_distinct_blocks_graph_sampled(state: tauri::State<AppState>, damselfly_instance: u64) -> Result<Vec<[f64; 2]>, String> {
    let mut viewer_lock = state.viewer.lock().unwrap();
    if let Some(viewer) = &mut *viewer_lock {
        Ok(viewer
            .damselflies
            .get_mut(damselfly_instance as usize)
            .expect("[tauri::command::get_viewer_distinct_blocks_graph_sampled]: damselfly_instance not found: {damselfly_instance}")
            .get_distinct_blocks_graph_realtime_sampled())
    } else {
        Err("Viewer is not initialised".to_string())
    }
}

#[tauri::command]
fn get_viewer_largest_block_graph(state: tauri::State<AppState>, damselfly_instance: u64) -> Result<Vec<[f64; 2]>, String> {
    let mut viewer_lock = state.viewer.lock().unwrap();
    if let Some(viewer) = &mut *viewer_lock {
        Ok(viewer
            .damselflies
            .get_mut(damselfly_instance as usize)
            .expect("[tauri::command::get_viewer_largest_block_graph]: damselfly instance not found: {damselfly_instance}")
            .get_largest_block_graph())
    } else {
        Err("Viewer is not initialised".to_string())
    }
}

#[tauri::command]
fn get_viewer_largest_block_graph_sampled(state: tauri::State<AppState>, damselfly_instance: u64) -> Result<Vec<[f64; 2]>, String> {
    let mut viewer_lock = state.viewer.lock().unwrap();
    if let Some(viewer) = &mut *viewer_lock {
        Ok(viewer
            .damselflies
            .get_mut(damselfly_instance as usize)
            .expect("[tauri::command::get_viewer_largest_block_graph_sampled]: damselfly_instance not found: {damselfly_instance}")
            .get_largest_block_graph_realtime_sampled())
    } else {
        Err("Viewer is not initialised".to_string())
    }
}

#[tauri::command]
fn get_viewer_free_blocks_graph(state: tauri::State<AppState>, damselfly_instance: u64) -> Result<Vec<[f64; 2]>, String> {
    let mut viewer_lock = state.viewer.lock().unwrap();
    if let Some(viewer) = &mut *viewer_lock {
        Ok(viewer
            .damselflies
            .get_mut(damselfly_instance as usize)
            .expect("[tauri::command::get_viewer_free_blocks_graph]: damselfly_instance not found: {damselfly_instance}")
            .get_free_blocks_graph())
    } else {
        Err("Viewer is not initialised".to_string())
    }
}

#[tauri::command]
fn get_viewer_free_blocks_graph_sampled(state: tauri::State<AppState>, damselfly_instance: u64) -> Result<Vec<[f64; 2]>, String> {
    let mut viewer_lock = state.viewer.lock().unwrap();
    if let Some(viewer) = &mut *viewer_lock {
        Ok(viewer
            .damselflies
            .get_mut(damselfly_instance as usize)
            .expect("[tauri::command::get_viewer_free_blocks_graph_sampled]: damselfly_instance not found: {damselfly_instance}")
            .get_free_blocks_graph_realtime_sampled())
    } else {
        Err("Viewer is not initialised".to_string())
    }
}

#[tauri::command]
fn get_viewer_map(state: tauri::State<AppState>, damselfly_instance: u64) -> Result<Vec<MemoryStatus>, String> {
    let mut viewer_lock = state.viewer.lock().unwrap();
    if let Some(viewer) = &mut *viewer_lock {
        Ok(viewer
            .damselflies
            .get_mut(damselfly_instance as usize)
            .expect("[tauri::command::get_viewer_map]: damselfly instance not found: {damselfly_instance}")
            .get_map_full_nosync())
    } else {
        Err("Viewer is not initialised".to_string())
    }
}

#[tauri::command]
fn get_viewer_map_full_at(state: tauri::State<AppState>, damselfly_instance: u64, timestamp: usize) -> Result<Vec<MemoryStatus>, String> {
    let mut viewer_lock = state.viewer.lock().unwrap();
    if let Some(viewer) = &mut *viewer_lock {
        Ok(viewer
            .damselflies
            .get_mut(damselfly_instance as usize)
            .expect("[tauri::command::get_viewer_map_full_at]: damselfly_instance not found: {damselfly_instance}")
            .get_map_full_at_nosync(timestamp))
    } else {
        Err("Viewer is not initialised".to_string())
    }
}

#[tauri::command]
fn get_viewer_map_full_at_colours(
    damselfly_instance: u64,
    state: tauri::State<AppState>,
    timestamp: u64,
    truncate_after: u64,
) -> Result<(u64, Vec<(i64, u64)>), String> {
    eprintln!("[tauri::get_viewer_map_full_at_colours]: timestamp: {timestamp}");
    let mut viewer_lock = state.viewer.lock().unwrap();
    if let Some(viewer) = &mut *viewer_lock {
        Ok(viewer
            .damselflies
            .get_mut(damselfly_instance as usize)
            .expect("[tauri::command::get_viewer_map_full_at_colours]: damselfly_instance not found: {damselfly_instance}")
            .get_map_full_at_nosync_colours_truncate(timestamp, truncate_after))
    } else {
        Err("Viewer is not initialised".to_string())
    }
}

#[tauri::command]
fn get_viewer_map_full_at_colours_realtime_sampled(
    damselfly_instance: u64,
    state: tauri::State<AppState>,
    timestamp: u64,
    truncate_after: u64,
) -> Result<(u64, Vec<(i64, u64)>), String> {
    eprintln!("[tauri::get_viewer_map_full_at_colours_realtime_sampled]: realtime_timestamp: {timestamp}");
    let mut viewer_lock = state.viewer.lock().unwrap();
    if let Some(viewer) = &mut *viewer_lock {
        let res = viewer
            .damselflies
            .get_mut(damselfly_instance as usize)
            .expect("[tauri::command::get_viewer_map_full_at_colours]: damselfly_instance not found: {damselfly_instance}")
            .get_map_full_at_nosync_colours_truncate_realtime_sampled(timestamp, truncate_after);
        eprintln!("[tauri::get_viewer_map_full_at_colours_realtime_sampled]: realtime sampled size: {}", res.1.len());
        Ok(res)
    } else {
        Err("Viewer is not initialised".to_string())
    }
}

#[tauri::command]
fn set_block_size(state: tauri::State<AppState>, damselfly_instance: u64, new_block_size: u64) -> Result<(), String> {
    let mut viewer_lock = state.viewer.lock().unwrap();
    if let Some(viewer) = &mut *viewer_lock {
        viewer
        .damselflies
        .get_mut(damselfly_instance as usize)
        .expect("[tauri::command::set_block_size]: damselfly_instance not found: {damselfly_instance}")
        .set_map_block_size(new_block_size as usize);
        Ok(())
    } else {
        Err("Viewer is not initialised".to_string())
    }
}

#[tauri::command]
fn get_operation_log(state: tauri::State<AppState>, damselfly_instance: u64) -> Result<Vec<String>, String> {
    let mut viewer_lock = state.viewer.lock().unwrap();
    if let Some(viewer) = &mut *viewer_lock {
        Ok(viewer
            .damselflies
            .get_mut(damselfly_instance as usize)
            .expect("[tauri::command::get_operation_log]: damselfly_instance not found: {damselfly_instance}")
            .get_operation_history()
            .iter()
            .take(128)
            .map(|update| update.to_string())
            .collect())
    } else {
        Err("Viewer is not initialised".to_string())
    }
}

#[tauri::command]
fn get_callstack(state: tauri::State<AppState>, damselfly_instance: u64) -> Result<String, String> {
    let mut viewer_lock = state.viewer.lock().unwrap();
    if let Some(viewer) = &mut *viewer_lock {
        Ok(viewer
            .damselflies
            .get_mut(damselfly_instance as usize)
            .expect("[tauri::command::get_callstack]: damselfly_instance not found: {damselfly_instance}")
            .get_current_operation().get_callstack().to_string())
    } else {
        Err("Viewer is not initialised".to_string())
    }
}

#[tauri::command]
fn query_block(
    damselfly_instance: u64,
    state: tauri::State<AppState>,
    address: usize,
    timestamp: usize,
) -> Result<Vec<MemoryUpdateType>, String> {
    let mut viewer_lock = state.viewer.lock().unwrap();
    if let Some(viewer) = &mut *viewer_lock {
        let mut updates = viewer
        .damselflies
        .get_mut(damselfly_instance as usize)
        .expect("[tauri::command::query_block]: damselfly_instance not found: {damselfly_instance}")
        .query_block(address, timestamp);
        eprintln!("[Tauri::query_block]: updates.len: {}", updates.len());
        updates.sort_by_key(|next| std::cmp::Reverse(next.get_timestamp()));
        updates.reverse();
        Ok(updates)
    } else {
        Err("Viewer is not initialised".to_string())
    }
}

#[tauri::command]
fn query_block_realtime(
    state: tauri::State<AppState>,
    damselfly_instance: u64,
    address: usize,
    timestamp: usize,
) -> Result<Vec<MemoryUpdateType>, String> {
    let mut viewer_lock = state.viewer.lock().unwrap();
    if let Some(viewer) = &mut *viewer_lock {
        let mut updates = viewer
        .damselflies
        .get_mut(damselfly_instance as usize)
        .expect("[tauri::command::query_block_realtime]: damselfly_instance not found: {damselfly_instance}")
        .query_block_realtime(address, timestamp);
        eprintln!("[Tauri::query_block_realtime]: damselfly_instance: {} address: {} timestamp: {} updates.len: {}", damselfly_instance, address, timestamp, updates.len());
        updates.sort_by_key(|next| std::cmp::Reverse(next.get_timestamp()));
        updates.reverse();
        Ok(updates)
    } else {
        Err("Viewer is not initialised".to_string())
    }
}

#[tauri::command]
fn get_pool_list(state: tauri::State<AppState>) -> Result<Vec<String>, String> {
    let mut viewer_lock = state.viewer.lock().unwrap();
    if let Some(viewer) = &mut *viewer_lock {
        return Ok(viewer
        .damselflies
        .iter()
        .map(|damselfly| String::from(damselfly.get_name()))
        .collect());
    } else {
        Err("Viewer is not initialised".to_string())
    }
}

mod tests {
    use damselfly3::damselfly::viewer::damselfly_viewer::DamselflyViewer;
    use crate::get_viewer_map_full_at_colours;

    #[test]
    fn benchmark() {
        let mut damselfly_viewer = DamselflyViewer::new("/home/signal/dev/trace.log", "/home/signal/dev/threadxApp", 1000);
        let malloc_query = damselfly_viewer.damselflies[1].query_block(3782706500, 12345);
        let cpp_query = damselfly_viewer.damselflies[0].query_block(3782706500, 12345);
        let map = damselfly_viewer.damselflies[1].get_map_full_at_nosync_colours_truncate(12345, 256);

        let realtime_sampled_map = damselfly_viewer.damselflies[0].get_map_full_at_nosync_colours_truncate(0, 256);
        eprintln!("done");
    }
}
