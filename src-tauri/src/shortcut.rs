extern crate cocoa;
extern crate objc;

use cocoa::{appkit::NSApp, base::{id, nil}, foundation::{NSAutoreleasePool, NSNotFound, NSRange, NSRect}};
use enigo::{
    Direction::{Click, Press, Release}, Enigo, Key, Keyboard, Settings
};
use std::{error::Error, sync::{Arc, Mutex}, thread, time::Duration};
use tauri::{App, AppHandle, Manager, PhysicalPosition, State};
use objc::{msg_send, runtime::Class, sel, sel_impl};

use crate::{
    clipboard,
    store::{RecordStore, RecordType},
};

struct AwakeState {
    ns_app_pid: Option<i32>
}

fn get_current_app_pid() -> Option<i32> {
    unsafe {
        let pool = NSAutoreleasePool::new(nil);

        let workspace: id = msg_send![Class::get("NSWorkspace").unwrap(), sharedWorkspace];
        let active_app: id = msg_send![workspace, frontmostApplication];

        if active_app != nil {
            pool.drain();
            return Some(msg_send![active_app, processIdentifier]);
        }
        pool.drain();
        None
    }
}

fn activate_window(app_pid: i32) -> bool {
    unsafe {
        let pool = NSAutoreleasePool::new(nil);
        let app: id = msg_send![Class::get("NSRunningApplication").unwrap(), runningApplicationWithProcessIdentifier: app_pid];
        if app != nil {
            let _: () = msg_send![app, activateWithOptions: 1];
            pool.drain();
            return true;
        }
        pool.drain();
        false
    }
}

fn get_cursor_position() -> Option<PhysicalPosition<i32>> {
    unsafe {
        let pool = NSAutoreleasePool::new(nil);
        let app: id = NSApp();
        if app == nil {
            println!("NSApp is nil");
            pool.drain();
            return None;
        }

        let key_window: id = msg_send![app, keyWindow];
        if key_window == nil {
            println!("keyWindow is nil");
            pool.drain();
            return None;
        }

        let first_responder: id = msg_send![key_window, firstResponder];
        if first_responder == nil {
            println!("firstResponder is nil");
            pool.drain();
            return None;
        }

        let is_text_view: bool = msg_send![first_responder, isKindOfClass: Class::get("NSTextView").unwrap()];
        let is_text_field: bool = msg_send![first_responder, isKindOfClass: Class::get("NSTextField").unwrap()];
        let is_combo_box: bool = msg_send![first_responder, isKindOfClass: Class::get("NSComboBox").unwrap()];

        if is_text_view {
            let selected_range: NSRange = msg_send![first_responder, selectedRange];
            if selected_range.location == NSNotFound as u64 {
                println!("selectedRange is NSNotFound");
                pool.drain();
                return None;
            }

            let rect: NSRect = msg_send![first_responder, firstRectForCharacterRange: selected_range];
            let screen_rect: NSRect = msg_send![first_responder, convertRectToScreen: rect];
            pool.drain();
            return Some(PhysicalPosition::new(screen_rect.origin.x as i32, screen_rect.origin.y as i32));
        } else if is_text_field || is_combo_box {
            let selected_range: NSRange = msg_send![first_responder, currentEditor];
            if selected_range.location == NSNotFound as u64 {
                println!("selectedRange is NSNotFound");
                pool.drain();
                return None;
            }

            let rect: NSRect = msg_send![first_responder, firstRectForCharacterRange: selected_range];
            let screen_rect: NSRect = msg_send![first_responder, convertRectToScreen: rect];
            pool.drain();
            return Some(PhysicalPosition::new(screen_rect.origin.x as i32, screen_rect.origin.y as i32));
        } else {
            println!("firstResponder is not a supported text input control");
            pool.drain();
            return None;
        }
    }
}

fn paste() {
    let mut enigo = Enigo::new(&Settings::default()).unwrap();
    enigo.key(Key::Meta, Press).unwrap();
    enigo.key(Key::Unicode('v'), Click).unwrap();
    thread::sleep(Duration::from_millis(500));
    enigo.key(Key::Meta, Release).unwrap();
}

#[cfg(desktop)]
pub fn init(app: &App) -> Result<(), Box<dyn Error>> {
    use std::sync::{Arc, Mutex};

    use tauri::{Emitter, Manager};
    use tauri_plugin_global_shortcut::{Builder, Code, Modifiers};
    use tauri_plugin_positioner::{Position, WindowExt};

    use crate::store::RecordStore;

    let awake_state = Mutex::new(AwakeState {ns_app_pid: None});
    app.handle().manage(awake_state);
    app.handle().plugin(
        Builder::new()
            .with_shortcuts(["ctrl+v"])?
            .with_handler(move |app_handle, shortcut, _event| {
                if shortcut.mods.contains(Modifiers::CONTROL) && shortcut.key == Code::KeyV {
                    let main_window = app_handle.get_webview_window("main").unwrap();
                    if !main_window.is_visible().unwrap() {
                        if let Some(cursor_position) = get_cursor_position() {
                            main_window.set_position(cursor_position).unwrap();
                        }else {
                            main_window.move_window(Position::Center).unwrap();
                        }

                        let awake_state = app_handle.state::<Mutex<AwakeState>>();
                        awake_state.lock().unwrap().ns_app_pid = get_current_app_pid();
                        
                        main_window.show().unwrap();
                        main_window.set_focus().unwrap();

                        let store = app_handle.state::<Arc<RecordStore>>();
                        app_handle.emit("fill-records", &store.get_records().unwrap_or_default()).unwrap();
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

        if let Some(ns_app_pid) = app_handle.state::<Mutex<AwakeState>>().lock().unwrap().ns_app_pid {
            activate_window(ns_app_pid);
            paste();
        }
    }
}
