use rfd::FileHandle;

pub enum CustomEvent {
    LoadDialog,
    SaveDialog(Option<FileHandle>),
    Quit,
}
