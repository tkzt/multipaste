use macos_clipboard::{Clipboard, ClipboardManager};
use std::sync::Arc;
use std::thread;

fn main() {
    // 创建剪贴板管理器
    let clipboard_manager = ClipboardManager::new();

    // 获取剪贴板实例
    let clipboard: Arc<dyn Clipboard> = Arc::new(clipboard_manager.get_clipboard());

    // 监听剪贴板变化并触发操作
    clipboard.on_change(|contents| {
        println!("剪贴板内容已更改：{}", contents);
        // 在这里执行你想要的操作
    });

    // 阻塞主线程，保持监听
    thread::park();
}