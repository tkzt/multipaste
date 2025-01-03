extern crate cocoa;
extern crate objc;

use accessibility::{AXAttribute, AXUIElement};
use accessibility_sys::{
    kAXFocusedWindowAttribute, kAXRaiseAction, kAXWindowsAttribute, AXError,
    AXUIElementCopyAttributeValues, AXUIElementCreateApplication, AXUIElementPerformAction,
    AXUIElementRef,
};
use cocoa::{
    appkit::NSApplicationActivationOptions::NSApplicationActivateIgnoringOtherApps,
    base::{id, nil},
    foundation::NSAutoreleasePool,
};
use core_foundation::{
    array::{CFArrayGetCount, CFArrayGetValueAtIndex},
    base::TCFType,
    string::CFString,
};
use core_graphics::display::CGWindowID;
use log::{info, warn};
use objc::{msg_send, runtime::Class, sel, sel_impl};

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    pub fn _AXUIElementGetWindow(el: AXUIElementRef, id: &mut CGWindowID) -> AXError;
}

pub struct WindowInfo {
    app_pid: i32,
    window_id: u32,
}

pub fn get_active_window_info() -> Option<WindowInfo> {
    unsafe {
        let _pool = NSAutoreleasePool::new(nil);
        let workspace: id = msg_send![Class::get("NSWorkspace").unwrap(), sharedWorkspace];
        let active_app: id = msg_send![workspace, frontmostApplication];
        if active_app != nil {
            let app_pid: i32 = msg_send![active_app, processIdentifier];
            let app_element = AXUIElement::application(app_pid);
            let Some(focused_window) = app_element
                .attribute(&AXAttribute::new(&CFString::from_static_string(
                    kAXFocusedWindowAttribute,
                )))
                .map(|el| el.downcast_into::<AXUIElement>())
                .ok()
                .flatten()
            else {
                warn!("Failed to get focused window");
                return None;
            };
            let mut window_id: u32 = 0;
            _AXUIElementGetWindow(focused_window.as_concrete_TypeRef(), &mut window_id);
            info!("Active app pid: {}, window id: {}", app_pid, window_id);
            return Some(WindowInfo { app_pid, window_id });
        } else {
            warn!("None active app found.")
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
        let _: () = msg_send![app, activateWithOptions: NSApplicationActivateIgnoringOtherApps];
        if app == nil {
            warn!("Failed to get the running application");
            return;
        }

        let app_element = AXUIElementCreateApplication(window_info.app_pid);
        let mut window_list_ref = std::ptr::null();
        AXUIElementCopyAttributeValues(
            app_element,
            CFString::new(kAXWindowsAttribute).as_concrete_TypeRef(),
            0,
            9999999,
            &mut window_list_ref,
        );
        if !window_list_ref.is_null() {
            let window_count = CFArrayGetCount(window_list_ref);
            if window_count == 0 {
                warn!("None matched window found.");
                return;
            }
            for i in 0..window_count {
                let mut window_id: u32 = 0;
                let window_ref =
                    CFArrayGetValueAtIndex(window_list_ref, i as isize) as AXUIElementRef;
                _AXUIElementGetWindow(window_ref, &mut window_id);

                if window_id == window_info.window_id {
                    AXUIElementPerformAction(
                        window_ref,
                        CFString::new(kAXRaiseAction).as_concrete_TypeRef(),
                    );
                    break;
                }
            }
        } else {
            warn!("Failed to get window list.")
        }
    }
}
