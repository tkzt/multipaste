// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod awake;
mod clipboard;
mod conf;
mod ns;
mod store;
mod tray;
mod windows;

use tauri::{ActivationPolicy, App, Window, WindowEvent};
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_log::{Target, TargetKind};

fn setup(app: &mut App) -> std::result::Result<(), Box<dyn std::error::Error>> {
    // to hide icon in dock
    app.set_activation_policy(ActivationPolicy::Accessory);

    tray::init(app);
    conf::init(app)?;
    let store = store::init(app)?;
    awake::init(app)?;
    clipboard::init(store);

    Ok(())
}

fn on_window_event(window: &Window, event: &WindowEvent) {
    match event {
        WindowEvent::Focused(focused) => {
            if !focused {
                window.close().unwrap();
            }
        }
        _ => (),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_positioner::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::LogDir { file_name: None }),
                    Target::new(TargetKind::Webview),
                ])
                .build(),
        )
        .on_window_event(on_window_event)
        .setup(setup)
        .invoke_handler(tauri::generate_handler![
            store::filter_records,
            store::pin_record,
            store::unpin_record,
            store::delete_record,
            awake::copy_record,
            conf::get_config,
            conf::update_auto_start,
            conf::update_max_items,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
