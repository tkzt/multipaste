use enigo::{Enigo, Mouse, Settings};
use tauri::{Manager, PhysicalPosition};

#[tauri::command]
pub fn search_focus() {
  
}


#[tauri::command]
pub fn awake(app_handle: tauri::AppHandle) {
  let main_window = app_handle.get_window("main").unwrap();
  let (x, y) = Enigo::new(&Settings::default()).unwrap().location().unwrap();
  let _ = main_window.set_position(PhysicalPosition::new(x, y));
  let _ = main_window.show();
  let _ = main_window.set_focus();
}