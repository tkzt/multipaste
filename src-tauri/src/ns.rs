extern crate cocoa;
extern crate objc;

use std::ptr;

use accessibility_sys::{kAXErrorSuccess, kAXFocusedWindowAttribute, kAXRaiseAction, kAXTrustedCheckOptionPrompt, kAXWindowsAttribute, AXError, AXIsProcessTrustedWithOptions, AXUIElementCopyAttributeValue, AXUIElementCopyAttributeValues, AXUIElementCreateApplication, AXUIElementPerformAction, AXUIElementRef};
use cocoa::{
    appkit::NSApplicationActivationOptions::NSApplicationActivateIgnoringOtherApps, base::{id, nil}, foundation::NSAutoreleasePool
};
use core_foundation::{array::{CFArrayGetCount, CFArrayGetValueAtIndex}, base::{CFRelease, TCFType, TCFTypeRef}, dictionary::{CFDictionaryAddValue, CFDictionaryCreateMutable}, number::kCFBooleanTrue, string::CFString};
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

pub fn check_accessibility_trusted() -> bool {
    unsafe {
        let options = CFDictionaryCreateMutable(
            ptr::null_mut(),
            1, 
            std::ptr::null(), std::ptr::null()
        );
        CFDictionaryAddValue(options, kAXTrustedCheckOptionPrompt.as_void_ptr(), kCFBooleanTrue.as_void_ptr());
        let is_allowed = AXIsProcessTrustedWithOptions(options);
        CFRelease(options as *const _);
        is_allowed
    }
}

pub fn get_active_window_info() -> Option<WindowInfo> {
    unsafe {
        let _pool = NSAutoreleasePool::new(nil);
        let workspace: id = msg_send![Class::get("NSWorkspace").unwrap(), sharedWorkspace];
        let active_app: id = msg_send![workspace, frontmostApplication];
        if active_app != nil {
            let app_pid: i32 = msg_send![active_app, processIdentifier];
            let app_element = AXUIElementCreateApplication(app_pid);
            let mut focused_window = ptr::null();
            if AXUIElementCopyAttributeValue(
                app_element,
                CFString::from_static_string(kAXFocusedWindowAttribute).as_concrete_TypeRef(),
                &mut focused_window
            ) == kAXErrorSuccess {
                let mut window_id: u32 = 0;
                _AXUIElementGetWindow(
                    focused_window as AXUIElementRef,
                    &mut window_id
                );
                if window_id > 0 {
                    info!("Focused window: {}", window_id);
                    return Some(WindowInfo {
                        app_pid,
                        window_id,
                    })
                } else {
                    warn!("Failed to get focused window id");
                }
            } else {
                warn!("Failed to copy attribute value {}", kAXFocusedWindowAttribute)
            }
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
                let window_ref = CFArrayGetValueAtIndex(window_list_ref, i as isize) as AXUIElementRef;
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