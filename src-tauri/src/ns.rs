extern crate cocoa;
extern crate objc;

use accessibility::{AXUIElement, AXUIElementAttributes};
use cocoa::{
    appkit::NSApplicationActivationOptions::NSApplicationActivateIgnoringOtherApps, base::{id, nil}, foundation::NSAutoreleasePool
};
use core_foundation::boolean::CFBoolean;
use objc::{msg_send, runtime::Class, sel, sel_impl};

pub struct WindowInfo {
    app_pid: i32,
    window_idx: usize,
}

pub fn get_active_window_info() -> Option<WindowInfo> {
    unsafe {
        let _pool = NSAutoreleasePool::new(nil);
        let workspace: id = msg_send![Class::get("NSWorkspace").unwrap(), sharedWorkspace];
        let active_app: id = msg_send![workspace, frontmostApplication];
        if active_app != nil {
            let app_pid: i32 = msg_send![active_app, processIdentifier];
            let app_element = AXUIElement::application(app_pid);
            let windows = app_element.windows().unwrap();
            for (window_idx, window) in windows.iter().enumerate() {
                if let Ok(is_main_window) = window.main() {
                    if is_main_window == CFBoolean::true_value() {
                        return Some(WindowInfo {
                            app_pid,
                            window_idx
                        })
                    }
                } 
            }
        }
        None
    }
}

pub fn activate_window(window_info: &WindowInfo) {
    unsafe {
        let _pool = NSAutoreleasePool::new(nil);
        let app: id = msg_send![
            Class::get("NSRunningApplication").unwrap(),
            runningApplicationWithProcessIdentifier: window_info.app_pid
        ];
        if app == nil {
            return;
        }
        
        let _: () = msg_send![app, activateWithOptions: NSApplicationActivateIgnoringOtherApps];
        let app_element = AXUIElement::application(window_info.app_pid);
        let windows = app_element.windows().unwrap();
        
        for (window_idx, window) in windows.iter().enumerate() {
            if window_idx == window_info.window_idx {
                window.set_main(CFBoolean::true_value()).unwrap();
                break;
            }
        }
    }
}