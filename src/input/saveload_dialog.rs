use std::{future::Future, path::PathBuf};

use rfd::{AsyncFileDialog, FileDialog, FileHandle};

use crate::constants::DEFAULT_SAVELOAD_PATH_STR;

// TODO: make async
pub fn select_load_file() -> Option<PathBuf> {
    FileDialog::new()
        .add_filter("ron", &["ron"])
        .set_directory(DEFAULT_SAVELOAD_PATH_STR)
        .pick_file()
}

pub fn select_save_file_async() -> impl Future<Output = Option<FileHandle>> {
    AsyncFileDialog::new()
        .add_filter("ron", &["ron"])
        .set_directory(DEFAULT_SAVELOAD_PATH_STR)
        .save_file()
}
