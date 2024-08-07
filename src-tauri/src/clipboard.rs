use clippers::Clipboard;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::store::{RecordStore, RecordType};

fn read_clipboard(store: &RecordStore) -> Result<(), Box<dyn std::error::Error>> {
    let mut clipboard = Clipboard::get();
    match clipboard.read() {
        Some(clippers::ClipperData::Text(text)) => {
            store.save(&RecordType::Text, &text.to_string(), None)?;
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

pub fn listen(store: Arc<Mutex<RecordStore>>) {
    thread::spawn(move || loop {
        let store = store.lock().unwrap();
        let read_res = read_clipboard(&store);
        if let Err(err) = read_res {
            eprintln!("Error reading clipboard: {}", err);
        }
        thread::sleep(Duration::from_millis(500));
    });
}
