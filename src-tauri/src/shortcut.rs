use std::{thread, time::Duration};
use enigo::{Enigo, Key, Keyboard, Settings};

fn check_key_pressed() {
  let enigo = Enigo::new(&Settings::default()).unwrap();
  if enigo.key(Key::Control) && enigo.is_key_pressed(Key::Layout('v')) {
    println!("Ctrl + V pressed");
  }
}

pub fn listen() {
    thread::spawn(move || loop {
        check_key_pressed();
        thread::sleep(Duration::from_millis(100));
    });
}
