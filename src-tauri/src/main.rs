// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod clipboard;
mod setup;
mod shortcut;

use tauri::{
    Manager, SystemTray, SystemTrayEvent, SystemTrayMenu
};
use shortcut::{search_focus, awake};


fn main() {
    let tray_menu = SystemTrayMenu::new();
    tauri::Builder::default()
        .system_tray(SystemTray::new().with_menu(tray_menu))
        .on_system_tray_event(|app, event| {
            match event {
                SystemTrayEvent::LeftClick { position, .. } | SystemTrayEvent::RightClick { position, .. } => {
                    let window = app.get_window("settings").unwrap();
                    let _ = window.set_position(position);
                    let _ = window.show();
                    let _ = window.set_focus();
                },
                _ => {}
            }
        })
        .setup(setup::init)
        .invoke_handler(tauri::generate_handler![search_focus, awake])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
