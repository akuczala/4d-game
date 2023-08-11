use std::{future::Future, path::Path, thread};

use futures_lite::future;
use glium::glutin::event_loop::EventLoopProxy;
use rfd::{AsyncFileDialog, FileHandle};

use crate::{
    config::{Config, LevelConfig, LoadLevelConfig},
    constants::DEFAULT_SAVELOAD_PATH_STR,
    utils::ValidDimension,
    vector::VecIndex,
};

use super::custom_events::CustomEvent;

// TODO: make async
fn select_load_file_async() -> impl Future<Output = Option<FileHandle>> {
    AsyncFileDialog::new()
        .add_filter("ron files", &["ron"])
        .set_directory(DEFAULT_SAVELOAD_PATH_STR)
        .pick_file()
}

fn select_save_file_async(dim: VecIndex) -> impl Future<Output = Option<FileHandle>> {
    AsyncFileDialog::new()
        .add_filter(
            &format!("{}d level files", dim),
            &[&save_file_extension(dim)],
        )
        .set_directory(DEFAULT_SAVELOAD_PATH_STR)
        // For some reason macos repeats the extension for the default name when this is "Untitled"
        // TODO: when the user includes the extension in the file name, it is duplicated
        .set_file_name("level")
        .save_file()
}

/// launches a thread that opens a save file dialog
pub fn request_save(dim: VecIndex, event_loop_proxy: &EventLoopProxy<CustomEvent>) {
    let dialog = select_save_file_async(dim);

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

/// launches a thread that opens a load file dialog
pub fn request_load(event_loop_proxy: &EventLoopProxy<CustomEvent>) {
    let dialog = select_load_file_async();
    let event_loop_proxy = event_loop_proxy.clone();
    thread::spawn(move || {
        future::block_on(async move {
            let file = dialog.await;
            event_loop_proxy
                .send_event(CustomEvent::LoadDialog(file))
                .ok();
        })
    });
}

fn save_file_extension(dim: VecIndex) -> String {
    format!("{}d.ron", dim)
}

fn get_file_dimension(path: &Path) -> Result<ValidDimension, String> {
    match path.to_str().unwrap_or_default() {
        s if s.ends_with(&save_file_extension(3)) => Ok(ValidDimension::Three),
        s if s.ends_with(&save_file_extension(4)) => Ok(ValidDimension::Four),
        s => Err(format!("Invalid file name {}.", s)),
    }
}

pub fn set_load_file_in_config(config: &mut Config, file: &Path) -> Result<ValidDimension, String> {
    let d = get_file_dimension(file)?;
    let s = file.to_str().ok_or("Could not parse file name.")?;
    let level_cfg = Some(LoadLevelConfig {
        path: s.to_string(),
    });
    match d {
        ValidDimension::Three => {
            config.scene.load_3d = level_cfg;
        }
        ValidDimension::Four => {
            config.scene.load_4d = level_cfg;
        }
    }
    config.scene.level = LevelConfig::Load;
    Ok(d)
}
