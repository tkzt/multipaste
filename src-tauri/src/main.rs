// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod clipboard;
mod setup;
mod shortcut;

use tauri::{
    CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu,
    SystemTrayMenuItem, WindowEvent,
};
use tauri_plugin_positioner::{Position, WindowExt};

use tauri::{
    CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu,
    SystemTrayMenuItem, WindowEvent,
};
use tauri_plugin_positioner::{Position, WindowExt};

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

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
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
