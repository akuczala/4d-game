use std::{future::Future, path::PathBuf, thread};

use futures_lite::future;
use glium::glutin::event_loop::EventLoopProxy;
use rfd::{AsyncFileDialog, FileDialog, FileHandle};

use crate::constants::DEFAULT_SAVELOAD_PATH_STR;

use super::custom_events::CustomEvent;

// TODO: make async
pub fn select_load_file() -> Option<PathBuf> {
    FileDialog::new()
        .add_filter("ron", &["ron"])
        .set_directory(DEFAULT_SAVELOAD_PATH_STR)
        .pick_file()
}

fn select_save_file_async() -> impl Future<Output = Option<FileHandle>> {
    AsyncFileDialog::new()
        .add_filter("ron", &["ron"])
        .set_directory(DEFAULT_SAVELOAD_PATH_STR)
        .save_file()
}

/// launches a thread that opens a save file dialog
pub fn request_save(event_loop_proxy: &EventLoopProxy<CustomEvent>) {
    let dialog = select_save_file_async();

    let event_loop_proxy = event_loop_proxy.clone();
    // TODO: seems sily to use block on in a thread?
    thread::spawn(move || {
        future::block_on(async move {
            let file = dialog.await;
            event_loop_proxy
                .send_event(CustomEvent::SaveDialog(file))
                .ok();
        })
    });
}
