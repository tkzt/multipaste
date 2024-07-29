use crate::{clipboard, shortcut};
use tauri::{App, Manager};
#[cfg(target_os = "macos")]
use window_vibrancy::NSVisualEffectMaterial;

pub fn init(app: &mut App) -> std::result::Result<(), Box<dyn std::error::Error>> {
    for label in ["main", "settings"] {
        let window = app.get_window(label).unwrap();

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

        clipboard::listen();
        shortcut::listen();
    }
    Ok(())
}
