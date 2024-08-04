use clippers::Clipboard;
use std::thread;
use std::time::Duration;

fn read_clipboard() {
    let mut clipboard = Clipboard::get();
    match clipboard.read() {
        Some(clippers::ClipperData::Text(text)) => {
            println!("Clipboard text: {:?}", text);
        }

        Some(clippers::ClipperData::Image(image)) => {
            println!("Clipboard image: {}x{}", image.width(), image.height());
        }

        Some(data) => {
            println!("Clipboard data is unknown: {data:?}");
        }

        None => {
            println!("Clipboard is empty");
        }
    }
}

pub fn listen() {
    thread::spawn(move || loop {
        read_clipboard();
        thread::sleep(Duration::from_millis(500));
    });
}
