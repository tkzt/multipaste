extern crate cocoa;
extern crate objc;

use cocoa::{
    base::{id, nil},
    foundation::NSAutoreleasePool,
};
use objc::{ msg_send, runtime::Class, sel, sel_impl};

pub fn get_current_app_pid() -> Option<i32> {
  unsafe {
      let _pool = NSAutoreleasePool::new(nil);
      let workspace: id = msg_send![Class::get("NSWorkspace").unwrap(), sharedWorkspace];
      let active_app: id = msg_send![workspace, frontmostApplication];
      if active_app != nil {
          let pid: i32 = msg_send![active_app, processIdentifier];
          return Some(pid);
      }
      None
  }
}

pub fn activate_window(app_pid: i32) -> bool {
  unsafe {
      let _pool = NSAutoreleasePool::new(nil);
      let app: id = msg_send![Class::get("NSRunningApplication").unwrap(), runningApplicationWithProcessIdentifier: app_pid];
      if app != nil {
          let _: () = msg_send![app, activateWithOptions: 1];
          return true;
      }
      false
  }
}
