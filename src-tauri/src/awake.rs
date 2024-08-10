use enigo::{
    Direction::{Click, Press, Release},
    Enigo, Key, Keyboard, Settings,
};
use std::{
    error::Error,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use tauri::{App, AppHandle, Manager, State};

use crate::{
    clipboard, ns::activate_window, store::{RecordStore, RecordType}
};

struct AwakeState {
    ns_app_pid: Option<i32>,
}

fn paste() {
    let mut enigo = Enigo::new(&Settings::default()).unwrap();
    enigo.key(Key::Meta, Press).unwrap();
    thread::sleep(Duration::from_millis(370));
    enigo.key(Key::Unicode('v'), Click).unwrap();
    enigo.key(Key::Meta, Release).unwrap();
}

#[cfg(desktop)]
pub fn init(app: &App) -> Result<(), Box<dyn Error>> {
    use std::sync::{Arc, Mutex};

    use tauri::{Emitter, Manager};
    use tauri_plugin_global_shortcut::{Builder, Code, Modifiers};
    use tauri_plugin_positioner::{Position, WindowExt};

    use crate::{ns::get_current_app_pid, store::RecordStore};

    let awake_state = Mutex::new(AwakeState { ns_app_pid: None });
    app.handle().manage(awake_state);
    app.handle().plugin(
        Builder::new()
            .with_shortcuts(["ctrl+v"])?
            .with_handler(move |app_handle, shortcut, _event| {
                if shortcut.mods.contains(Modifiers::CONTROL) && shortcut.key == Code::KeyV {
                    let main_window = app_handle.get_webview_window("main").unwrap();
                    if !main_window.is_visible().unwrap() {
                        main_window.move_window(Position::Center).unwrap();

                        let awake_state = app_handle.state::<Mutex<AwakeState>>();
                        awake_state.lock().unwrap().ns_app_pid = get_current_app_pid();

                        main_window.show().unwrap();
                        main_window.set_focus().unwrap();

                        let store = app_handle.state::<Arc<RecordStore>>();
                        app_handle
                            .emit("fill-records", &store.get_records().unwrap_or_default())
                            .unwrap();
                    }
                }
            })
            .build(),
    )?;
    Ok(())
}

#[tauri::command]
pub fn copy_record(app_handle: AppHandle, store: State<Arc<RecordStore>>, id: u64) {
    let record = store.get_record(&id).unwrap();
    if record.record_type == RecordType::Text {
        let main_window = app_handle.get_webview_window("main").unwrap();
        let _ = main_window.hide();
        clipboard::write_text(&record.record_value);

        if let Some(ns_app_pid) = app_handle
            .state::<Mutex<AwakeState>>()
            .lock()
            .unwrap()
            .ns_app_pid
        {
            activate_window(ns_app_pid);
            thread::sleep(Duration::from_millis(100));
            paste();
        }
    }
}
