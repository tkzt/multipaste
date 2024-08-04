use std::error::Error;
use tauri::App;

#[cfg(desktop)]
pub fn init(app: &App) -> Result<(), Box<dyn Error>> {
  use tauri::Manager;
  use tauri_plugin_global_shortcut::{Modifiers, Builder, Code};
use tauri_plugin_positioner::{Position, WindowExt};

  app.handle().plugin(
    Builder::new()
    .with_shortcuts(["ctrl+v"])?
    .with_handler(move |app_handle, shortcut, _event| {
      if shortcut.mods.contains(Modifiers::CONTROL) && shortcut.key == Code::KeyV {
        let main_window = app_handle.get_webview_window("main").unwrap();
        let _ = main_window.move_window(Position::Center);
        let _ = main_window.show();
        let _ = main_window.set_focus();
      }
    })
    .build()
  )?;
  Ok(())
}