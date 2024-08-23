use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

use log::warn;
use tauri::App;
use tauri::AppHandle;
use tauri::Manager;

use serde::{Deserialize, Serialize};
use tauri::State;
use tauri_plugin_autostart::ManagerExt;

use crate::store::RecordStore;

const CONFIG_PATH: &str = "config.json";
const DEFAULT_MAX_ITEMS: u64 = 200;
const DEFAULT_CONFIG_STR: &str = r#"{
  "auto_start": true,
  "max_items": 200
}"#;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub auto_start: bool,
    pub max_items: u64,
}

fn get_config_path(app_handle: &AppHandle) -> PathBuf {
    let app_config_dir = app_handle.path().app_config_dir().unwrap();
    if !app_config_dir.exists() {
        std::fs::create_dir_all(&app_config_dir).unwrap();
    }
    app_config_dir.join(CONFIG_PATH)
}

#[tauri::command]
pub fn update_auto_start(
    auto_start: bool,
    app_handle: AppHandle,
    config: State<Mutex<Config>>,
) -> bool {
    if let Ok(mut config) = config.lock() {
        let autolaunch_manager = app_handle.autolaunch();
        if autolaunch_manager.is_enabled().is_ok_and(|v| v) {
            if !auto_start {
                if autolaunch_manager.disable().is_err() {
                    warn!("Failed to disable autolaunch");
                    return false;
                }
            }
        } else {
            if auto_start {
                if autolaunch_manager.enable().is_err() {
                    warn!("Failed to enable autolaunch");
                    return false;
                }
            }
        }
        config.auto_start = auto_start;
        let config_path = get_config_path(&app_handle);
        if let Ok(_) = dump_config(&config_path, &config) {
            return true;
        }
        warn!("Failed to dump config.");
    }
    false
}

#[tauri::command]
pub fn update_max_items(
    max_items: u64,
    app_handle: AppHandle,
    config: State<Mutex<Config>>,
    store: State<Arc<RecordStore>>,
) -> bool {
    if let Ok(mut config) = config.lock() {
        if max_items <= 0 || store.update_max_records_trigger(max_items).is_err() {
            return false;
        }
        config.max_items = max_items;
        let config_path = get_config_path(&app_handle);
        if let Ok(_) = dump_config(&config_path, &config) {
            return true;
        }
    }
    false
}

#[tauri::command]
pub fn get_config(app_handle: AppHandle) -> Config {
    let config_path = get_config_path(&app_handle);
    load_config(&config_path).unwrap()
}

pub fn load_config(config_path: &PathBuf) -> Result<Config, Box<dyn std::error::Error>> {
    let dump_default = || {
        let default_config = Config {
            auto_start: false,
            max_items: DEFAULT_MAX_ITEMS,
        };
        std::fs::write(config_path, DEFAULT_CONFIG_STR).unwrap();
        Ok(default_config)
    };
    if config_path.exists() {
        if let Ok(content) = std::fs::read_to_string(config_path) {
            let config: Config = serde_json::from_str(&content)?;
            return Ok(config);
        } else {
            warn!("Failed to read config file.");
        }
    }
    dump_default()
}

fn dump_config(config_path: &PathBuf, config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::write(
        config_path,
        serde_json::to_string(config).unwrap_or(DEFAULT_CONFIG_STR.to_string()),
    )?;
    Ok(())
}

pub fn init(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = get_config_path(&app.app_handle());
    let config = load_config(&config_path)?;
    let config_state = Mutex::new(config);
    app.manage(config_state);
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn test_01_load_config() {
        let config_path = Path::new(".").join(CONFIG_PATH);
        let config = load_config(&config_path).unwrap();
        assert_eq!(config.auto_start, false);
        assert_eq!(config.max_items, 200);
    }

    #[test]
    fn test_02_dump_config() {
        let config_path = Path::new(".").join(CONFIG_PATH);
        dump_config(
            &config_path,
            &Config {
                auto_start: true,
                max_items: 1000,
            },
        )
        .unwrap();

        let config = load_config(&config_path).unwrap();
        assert_eq!(config.auto_start, true);
        assert_eq!(config.max_items, 1000);
    }
}
