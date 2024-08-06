// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod clipboard;
mod shortcut;
mod store;
mod tray;

use tauri::{ActivationPolicy, App, Manager, Window, WindowEvent};
#[cfg(target_os = "macos")]
use window_vibrancy::NSVisualEffectMaterial;

fn setup(app: &mut App) -> std::result::Result<(), Box<dyn std::error::Error>> {
    for label in ["main", "settings"] {
        let window = app.app_handle().get_webview_window(label).unwrap();

        // to hide icon in dock
        #[cfg(target_os = "macos")]
        app.set_activation_policy(ActivationPolicy::Accessory);

        #[cfg(target_os = "macos")]
        window_vibrancy::apply_vibrancy(
            &window,
            NSVisualEffectMaterial::HudWindow,
            None,
            Some(12.0),
        )
        .expect("Unsupported platform! 'apply_vibrancy' is only supported on macOS");

        #[cfg(target_os = "windows")]
        window_vibrancy::apply_blur(&window, Some((18, 18, 18, 125)))
            .expect("Unsupported platform! 'apply_blur' is only supported on Windows");
    }

    tray::init(app);
    shortcut::init(app)?;
    let store = store::init(app)?;
    clipboard::listen(store);
    Ok(())
}

fn on_window_event(window: &Window, event: &WindowEvent) {
    match event {
        WindowEvent::Focused(focused) => {
            if !focused {
                let _ = window.hide();
            }
        }
        _ => (),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_positioner::init())
        .on_window_event(on_window_event)
        .setup(setup)
        .invoke_handler(tauri::generate_handler![store::get_clipboard_records])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
