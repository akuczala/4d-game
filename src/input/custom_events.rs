use std::path::PathBuf;

use rfd::FileHandle;

pub enum CustomEvent {
    LoadDialog(Option<FileHandle>),
    SaveDialog(Option<FileHandle>),
    Quit,
    SwapEngine,
    LoadLevel(PathBuf),
    NewLevel,
    RestartLevel,
}
