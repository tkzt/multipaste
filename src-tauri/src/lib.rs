// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod awake;
mod clipboard;
mod ns;
mod store;
mod tray;

use tauri::{ActivationPolicy, App, Manager, Window, WindowEvent};
use tauri_plugin_log::{Target, TargetKind};
use tauri_plugin_positioner::{Position, WindowExt};
use window_vibrancy::NSVisualEffectMaterial;

fn setup(app: &mut App) -> std::result::Result<(), Box<dyn std::error::Error>> {
    for label in ["main", "settings"] {
        let window = app.app_handle().get_webview_window(label).unwrap();

        // to hide icon in dock
        app.set_activation_policy(ActivationPolicy::Accessory);

        window_vibrancy::apply_vibrancy(
            &window,
            NSVisualEffectMaterial::HudWindow,
            None,
            Some(12.0),
        )
        .expect("Unsupported platform! 'apply_vibrancy' is only supported on macOS");
    }

    tray::init(app);
    let store = store::init(app)?;
    app.handle().manage(store.clone());
    clipboard::listen(store);
    awake::init(app)?;

    Ok(())
}

fn on_window_event(window: &Window, event: &WindowEvent) {
    match event {
        WindowEvent::Focused(focused) => {
            if !focused {
                if window.label() == "main" {
                    let _ = window.move_window(Position::Center);
                }
                let _ = window.hide();
            }
        }
        _ => (),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        // .plugin(tauri_plugin_autostart::init())
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
