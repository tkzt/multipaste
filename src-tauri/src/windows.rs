use tauri::{utils::config::WindowConfig, AppHandle, Result, WebviewUrl, WebviewWindow};
use window_vibrancy::NSVisualEffectMaterial;

fn gen_basic_config() -> WindowConfig {
  let mut config = WindowConfig::default();
  config.resizable = false;
  config.transparent = true;
  config.decorations = false;
  config.visible = false;

  config
}

fn create_window(app_handle: &AppHandle, config: &WindowConfig) -> Result<WebviewWindow> {
  let window = tauri::WebviewWindowBuilder::from_config(
      app_handle,
      &config
  ).unwrap().build()?;

  window_vibrancy::apply_vibrancy(
      &window,
      NSVisualEffectMaterial::HudWindow,
      None,
      Some(12.0),
  ).unwrap();

  Ok(window)
}

pub fn create_main_window(app_handle: &AppHandle) -> Result<WebviewWindow>{
  let mut config = gen_basic_config();
  config.title = "Multipaste".to_string();
  config.label = "main".to_string();
  config.width = 400_f64;
  config.min_height = Some(400_f64);
  config.url = WebviewUrl::App("/".into());

  create_window(app_handle, &config)
}

pub fn create_settings_window(app_handle: &AppHandle) -> Result<WebviewWindow>{
  let mut config = gen_basic_config();
  config.title = "Settings".to_string();
  config.label = "settings".to_string();
  config.width = 180_f64;
  config.height = 165_f64;
  config.y = Some(0_f64);
  config.url = WebviewUrl::App("/settings".into());

  create_window(app_handle, &config)
}