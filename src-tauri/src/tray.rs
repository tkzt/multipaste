use tauri::{
  tray::TrayIconEvent, App, Manager
};

pub fn init(app: &App) {
    let tray_icon = app.tray_by_id("multipaste-tray").unwrap();
    tray_icon.on_tray_icon_event(|tray_icon, event| match event {
      TrayIconEvent::Click { rect, .. } => {
          let window = tray_icon
              .app_handle()
              .get_webview_window("settings")
              .unwrap();
            let _ = window.set_position(rect.position);
            let _ = window.show();
            let _ = window.set_focus();
      }
      _ => {}
  })
}
