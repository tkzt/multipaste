// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod clipboard;
mod setup;
mod shortcut;

use tauri::{
    Manager, SystemTray, SystemTrayEvent, SystemTrayMenu,
};
use tauri_plugin_positioner::{Position, WindowExt};
use shortcut::search_focus;


fn main() {
    let tray_menu = SystemTrayMenu::new();
    tauri::Builder::default()
        .plugin(tauri_plugin_positioner::init())
        .on_system_tray_event(|app, event| {
            tauri_plugin_positioner::on_tray_event(app, &event);
            match event {
                SystemTrayEvent::RightClick { .. } => {
                        let window = app.get_window("settings").unwrap();
                        let _ = window.move_window(Position::TrayCenter);
                        let _ = window.show();
                        let _ = window.set_focus();
                },
                _ => {}
            }
        })
        .system_tray(SystemTray::new().with_menu(tray_menu))
        .setup(setup::init)
        .invoke_handler(tauri::generate_handler![search_focus])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
