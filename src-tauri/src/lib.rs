mod ble;
mod input;
mod persistence;
mod protocol;
mod state;

use ble::BleState;
use state::{new_shared_state, SharedAppState};
use std::sync::Arc;
use tauri::{Emitter, Manager};
use tokio::sync::Mutex;

type SharedBle = Arc<Mutex<BleState>>;

#[tauri::command]
async fn scan_ble_devices(ble: tauri::State<'_, SharedBle>) -> Result<Vec<ble::BleDeviceInfo>, String> {
    let ble = ble.lock().await;
    ble.scan_devices().await
}

#[tauri::command]
async fn connect_ble_device(
    address: String,
    ble: tauri::State<'_, SharedBle>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let ble_guard = ble.lock().await;
    ble_guard.connect(&address).await?;
    drop(ble_guard);
    persistence::save_last_device(&app_handle, &address)?;
    let _ = app_handle.emit("connection-changed", true);
    Ok(())
}

#[tauri::command]
async fn disconnect_ble_device(
    ble: tauri::State<'_, SharedBle>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let ble = ble.lock().await;
    ble.disconnect().await?;
    let _ = app_handle.emit("connection-changed", false);
    Ok(())
}

#[tauri::command]
async fn get_connection_status(ble: tauri::State<'_, SharedBle>) -> Result<bool, String> {
    let ble = ble.lock().await;
    Ok(ble.is_connected().await)
}

#[tauri::command]
fn get_last_device(app_handle: tauri::AppHandle) -> Result<Option<String>, String> {
    persistence::load_last_device(&app_handle)
}

#[tauri::command]
fn set_passthrough(
    enable_keyboard: bool,
    enable_mouse: bool,
    enable_video: bool,
    app_state: tauri::State<'_, SharedAppState>,
) -> Result<(), String> {
    let mut state = app_state.lock().map_err(|e| e.to_string())?;
    state.passthrough.enable_keyboard = enable_keyboard;
    state.passthrough.enable_mouse = enable_mouse;
    state.passthrough.enable_video = enable_video;
    Ok(())
}

#[tauri::command]
fn get_passthrough(
    app_state: tauri::State<'_, SharedAppState>,
) -> Result<state::PassthroughConfig, String> {
    let state = app_state.lock().map_err(|e| e.to_string())?;
    Ok(state.passthrough.clone())
}

#[tauri::command]
fn toggle_lock_mode(
    app_state: tauri::State<'_, SharedAppState>,
    app_handle: tauri::AppHandle,
) -> Result<bool, String> {
    let mut state = app_state.lock().map_err(|e| e.to_string())?;
    state.lock_mode = !state.lock_mode;
    let new_mode = state.lock_mode;
    drop(state);
    let _ = app_handle.emit("lock-mode-changed", new_mode);
    Ok(new_mode)
}

#[tauri::command]
fn get_lock_mode(app_state: tauri::State<'_, SharedAppState>) -> Result<bool, String> {
    let state = app_state.lock().map_err(|e| e.to_string())?;
    Ok(state.lock_mode)
}

#[tauri::command]
fn start_control(
    ble: tauri::State<'_, SharedBle>,
    app_state: tauri::State<'_, SharedAppState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let ble_arc = Arc::clone(&*ble);
    let state_arc = Arc::clone(&*app_state);
    input::start_hook(ble_arc, state_arc, app_handle)
}

#[tauri::command]
fn stop_control() -> Result<(), String> {
    input::stop_hook();
    Ok(())
}

#[tauri::command]
fn is_control_active() -> bool {
    input::is_running()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let ble_state = tauri::async_runtime::block_on(async {
                BleState::new().await.expect("Failed to initialize BLE")
            });
            let ble = Arc::new(Mutex::new(ble_state));
            let app_state = new_shared_state();

            app.manage(ble);
            app.manage(app_state);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            scan_ble_devices,
            connect_ble_device,
            disconnect_ble_device,
            get_connection_status,
            get_last_device,
            set_passthrough,
            get_passthrough,
            toggle_lock_mode,
            get_lock_mode,
            start_control,
            stop_control,
            is_control_active,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

