use enigo::{
    Direction::{Click, Press, Release},
    Enigo, Key, Keyboard, Settings,
};
use log::{info, warn};
use std::{
    error::Error,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use tauri::{App, AppHandle, Manager, State};

use crate::{
    clipboard,
    ns::{activate_window, get_active_window_info, insert_text_to_focused_element, WindowInfo},
    store::{RecordStore, RecordType},
    windows::create_main_window,
};

struct AwakeState {
    active_window: Option<WindowInfo>,
}

fn paste_once() -> bool {
    let Ok(mut enigo) = Enigo::new(&Settings::default()) else {
        warn!("Failed to initialize enigo for simulated paste");
        return false;
    };

    if enigo.key(Key::Meta, Press).is_err() {
        return false;
    }
    thread::sleep(Duration::from_millis(20));

    if enigo.key(Key::Unicode('v'), Click).is_err() {
        let _ = enigo.key(Key::Meta, Release);
        return false;
    }
    thread::sleep(Duration::from_millis(20));

    enigo.key(Key::Meta, Release).is_ok()
}

fn paste_with_retry() -> bool {
    for _ in 0..2 {
        if paste_once() {
            return true;
        }
        thread::sleep(Duration::from_millis(40));
    }
    false
}

pub fn init(app: &App) -> Result<(), Box<dyn Error>> {
    use std::sync::Mutex;

    use tauri::Manager;
    use tauri_plugin_global_shortcut::{Builder, Code, Modifiers};
    use tauri_plugin_positioner::{Position, WindowExt};

    let awake_state = Mutex::new(AwakeState {
        active_window: None,
    });
    app.handle().manage(awake_state);
    app.handle().plugin(
        Builder::new()
            .with_shortcuts(["ctrl+v"])?
            .with_handler(move |app_handle, shortcut, _event| {
                warn!("Shortcut pressed: {:?}", shortcut);
                if shortcut.mods.contains(Modifiers::CONTROL) && shortcut.key == Code::KeyV {
                    if let None = app_handle.get_webview_window("main") {
                        let active_window_info = get_active_window_info();
                        let main_window = create_main_window(app_handle);
                        if let Ok(main_window) = main_window {
                            if !main_window.is_visible().unwrap() {
                                let awake_state = app_handle.state::<Mutex<AwakeState>>();
                                awake_state.lock().unwrap().active_window = active_window_info;
                                main_window.center().unwrap();
                                main_window.move_window(Position::Center).unwrap();
                                main_window.show().unwrap();
                                main_window.set_focus().unwrap();
                            }
                        } else {
                            warn!("Failed to create main window.");
                        }
                    } else {
                        warn!("Main window already exists.");
                    }
                }
            })
            .build(),
    )?;
    Ok(())
}

#[tauri::command]
pub fn copy_record(app_handle: AppHandle, store: State<Arc<RecordStore>>, id: i32) {
    if let (Ok(record), Some(main_window)) =
        (store.get_record(&id), app_handle.get_webview_window("main"))
    {
        if let Ok(_) = main_window.close() {
            if record.record_type == RecordType::Text {
                info!("Copying text: {}", record.record_value);
                clipboard::write_text(&record.record_value);
            } else {
                let image_path = store.get_image_paste_path(&record);
                clipboard::write_image(&image_path);
            }

            if let Some(active_window) = &app_handle
                .state::<Mutex<AwakeState>>()
                .lock()
                .unwrap()
                .active_window
            {
                activate_window(active_window);

                if record.record_type == RecordType::Text
                    && insert_text_to_focused_element(&record.record_value)
                {
                    info!("Inserted text via AX focused element.");
                } else if !paste_with_retry() {
                    warn!("Failed to paste record through simulated shortcut.");
                }
            } else {
                warn!("Failed to get active window.");
            }
        } else {
            warn!("Failed to close main window.");
        }
    } else {
        warn!("Failed to get record or main window.");
    }
}
