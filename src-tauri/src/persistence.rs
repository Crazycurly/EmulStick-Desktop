use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Default, Serialize, Deserialize)]
struct Config {
    last_device_address: Option<String>,
}

fn config_path(app_handle: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create config dir: {}", e))?;
    Ok(dir.join("config.json"))
}

use tauri::Manager;

pub fn save_last_device(app_handle: &tauri::AppHandle, address: &str) -> Result<(), String> {
    let path = config_path(app_handle)?;
    let config = Config {
        last_device_address: Some(address.to_string()),
    };
    let json =
        serde_json::to_string_pretty(&config).map_err(|e| format!("Serialize error: {}", e))?;
    fs::write(&path, json).map_err(|e| format!("Failed to write config: {}", e))
}

pub fn load_last_device(app_handle: &tauri::AppHandle) -> Result<Option<String>, String> {
    let path = config_path(app_handle)?;
    if !path.exists() {
        return Ok(None);
    }
    let data = fs::read_to_string(&path).map_err(|e| format!("Failed to read config: {}", e))?;
    let config: Config =
        serde_json::from_str(&data).map_err(|e| format!("Failed to parse config: {}", e))?;
    Ok(config.last_device_address)
}
