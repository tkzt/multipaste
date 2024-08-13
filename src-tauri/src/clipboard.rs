use clippers::Clipboard;
use image::ImageOutputFormat;
use log::{error, warn};
use std::io::Cursor;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::store::RecordStore;

fn read_clipboard(store: &RecordStore) -> Result<(), Box<dyn std::error::Error>> {
    let mut clipboard = Clipboard::get();
    match clipboard.read() {
        Some(clippers::ClipperData::Text(clipper_text)) => {
            let text = clipper_text.to_string();
            if !text.trim().is_empty() {
                store.save_text(&text.to_string())?;
            }
        }

        Some(clippers::ClipperData::Image(image)) => {
            let mut img_bytes: Vec<u8> = Vec::new();
            image
                .write_to(&mut Cursor::new(&mut img_bytes), ImageOutputFormat::Png)
                .unwrap();
            store.save_image(&img_bytes)?;
        }

        _ => {
            warn!("Empty clipboard or unsupported data.");
        }
    }
    Ok(())
}

pub fn write_text(text: &str) {
    let mut clipboard = Clipboard::get();
    clipboard.write_text(text).unwrap();
}

pub fn write_image(image_path: PathBuf) {
    if image_path.exists() {
        let image = image::open(image_path).unwrap();
        let mut clipboard = Clipboard::get();
        clipboard
            .write_image(image.width(), image.height(), image.as_bytes())
            .unwrap();
    }
}

pub fn listen(store: Arc<RecordStore>) {
    thread::spawn(move || loop {
        let read_res = read_clipboard(&store);
        if let Err(err) = read_res {
            error!("Error reading clipboard: {}", err);
        }
        thread::sleep(Duration::from_millis(500));
    });
}
