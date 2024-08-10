use clippers::Clipboard;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::store::{RecordStore, RecordType};

fn read_clipboard(store: &RecordStore) -> Result<(), Box<dyn std::error::Error>> {
    let mut clipboard = Clipboard::get();
    match clipboard.read() {
        Some(clippers::ClipperData::Text(clipper_text)) => {
            let text = clipper_text.to_string();
            if !text.trim().is_empty() {
                store.save(&RecordType::Text, &text.to_string(), None)?;
            }
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
    Ok(())
}

pub fn write_text(text: &str) {
    let mut clipboard = Clipboard::get();
    let _ = clipboard.write_text(text);
}

pub fn listen(store: Arc<RecordStore>) {
    thread::spawn(move || loop {
        let read_res = read_clipboard(&store);
        if let Err(err) = read_res {
            eprintln!("Error reading clipboard: {}", err);
        }
        thread::sleep(Duration::from_millis(500));
    });
}
