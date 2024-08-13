use tauri::{tray::TrayIconEvent, App};
use tauri_plugin_positioner::{Position, WindowExt};

use crate::windows::create_settings_window;

pub fn init(app: &App) {
    let tray_icon = app.tray_by_id("multipaste-tray").unwrap();
    tray_icon.on_tray_icon_event(|tray_icon, event| {
        tauri_plugin_positioner::on_tray_event(tray_icon.app_handle(), &event);
        match event {
            TrayIconEvent::Click { .. } => {
                let settings_window = create_settings_window(tray_icon.app_handle());
                if let Ok(settings_window) = settings_window {
                    settings_window.move_window(Position::TrayCenter).unwrap();
                    settings_window.show().unwrap();
                    settings_window.set_focus().unwrap();
                }
            }
            _ => {}
        }
    })
}
