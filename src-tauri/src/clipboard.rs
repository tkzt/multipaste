use clipboard_rs::common::RustImage;
use clipboard_rs::{Clipboard, ClipboardContext, ClipboardHandler, ClipboardWatcher, ClipboardWatcherContext};
use image::ImageFormat;
use log::{error, warn};
use std::io::Cursor;
use std::thread;
use std::path::PathBuf;
use std::sync::Arc;

use crate::store::RecordStore;


pub struct ClipboardManager {
    ctx: ClipboardContext,
    store: Arc<RecordStore>
}

impl ClipboardManager {
	pub fn new(store: Arc<RecordStore>) -> Self {
		let ctx = ClipboardContext::new().unwrap();
		ClipboardManager { ctx, store }
	}
}

impl ClipboardHandler for ClipboardManager {
	fn on_clipboard_change(&mut self) {
		if let Ok(text) = self.ctx.get_text() {
            if !text.trim().is_empty() {
                if let Err(err) = self.store.save_text(&text.to_string()){
                    error!("Error saving text: {}", err);
                }
            }
        } else if let Ok(img) = self.ctx.get_image() {
            let mut img_bytes: Vec<u8> = Vec::new();
            if let Ok(img_data) = img.get_dynamic_image() {
                if let Ok(_) = img_data.write_to(&mut Cursor::new(&mut img_bytes), ImageFormat::Png) {
                    if let Err(err) = self.store.save_image(&img_bytes) {
                        error!("Error saving image: {}", err);
                    }
                } else {
                    warn!("Error writing image to buffer.");
                }
            }
        }
	}
}

pub fn write_text(text: &str) -> bool {
    let ctx = ClipboardContext::new().unwrap();
    if let Err(err) = ctx.set_text(text.to_string()) {
        error!("Error setting text: {}", err);
        return false;
    }
    return true;
}

pub fn write_image(image_path: &PathBuf) -> bool {
    let ctx = ClipboardContext::new().unwrap();
    if image_path.exists() {
        if let Ok(image_data) = RustImage::from_path(image_path.to_str().unwrap()) {
            if let Err(err) = ctx.set_image(image_data) {
                error!("Error setting image: {}", err);
            } else {
                return true;
            }
        } else {
            warn!("Error reading image data from path.");
        }
    } else {
        warn!("Image path {} does not exist.", image_path.display());
    }
    return false;
}

pub fn init(store: Arc<RecordStore>) {
    let manager = ClipboardManager::new(store);
    let mut watcher: ClipboardWatcherContext<ClipboardManager> = ClipboardWatcherContext::new().unwrap();
    watcher.add_handler(manager);
    thread::spawn(move || {
        watcher.start_watch();
    });
}
