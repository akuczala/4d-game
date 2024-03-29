use std::error;

use serde::{Deserialize, Serialize};

use crate::{constants::CONFIG_FILE_PATH_STR, draw::ViewportShape, vector::Field};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ViewConfig {
    pub height: Field,
    pub radius: Field,
    pub viewport_shape: ViewportShape,
    pub focal: Field,
    pub spin_speed: Field,
}
impl Default for ViewConfig {
    fn default() -> Self {
        Self {
            height: 0.5,
            radius: 0.6,
            viewport_shape: ViewportShape::Cylinder,
            focal: 1.0,
            spin_speed: 0.1,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct FuzzLinesConfig {
    pub face_num: usize,
    pub sky_num: usize,
    pub horizon_num: usize,
}
impl Default for FuzzLinesConfig {
    fn default() -> Self {
        Self {
            face_num: 500,
            sky_num: 1000,
            horizon_num: 500,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum LevelConfig {
    Level1,
    Test1,
    Test2,
    Empty,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct Level1Config {
    pub open_center: bool,
}
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SceneConfig {
    pub grid: bool,
    pub sky: bool,
    pub horizon: bool,
    pub stars: bool,
    pub level: LevelConfig,
    pub level_1: Option<Level1Config>,
}
impl Default for SceneConfig {
    fn default() -> Self {
        Self {
            grid: false,
            sky: false,
            horizon: false,
            stars: true,
            level: LevelConfig::Level1,
            level_1: None,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct EditorConfig {
    pub enabled: bool,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct Config {
    pub fuzz_lines: FuzzLinesConfig,
    pub view: ViewConfig,
    pub scene: SceneConfig,
    pub editor: EditorConfig,
}

type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

pub fn load_config_2() -> Result<Config> {
    let r1 = std::fs::read_to_string(CONFIG_FILE_PATH_STR)?;
    let r2 = toml::from_str::<Config>(&r1)?;
    Ok(r2)
}

pub fn load_config() -> Config {
    std::fs::read_to_string(CONFIG_FILE_PATH_STR)
        .map_err(|e| println!("Error loading config {}: {}", CONFIG_FILE_PATH_STR, e))
        .and_then(|s| {
            toml::from_str::<Config>(&s).map_err(|e| println!("Could not parse config file: {}", e))
        })
        .map_err(|_| println!("Using default config"))
        .unwrap_or(Config::default())
}

pub fn get_config() -> Config {
    load_config_2()
        .map_err(|e| println!("Error loading config: {}", e))
        .unwrap_or(Config::default())
}

pub fn save_config(config: Config) -> std::result::Result<(), ()> {
    toml::to_string_pretty(&config)
        .map_err(|e| println!("Could not serialize config {:?}: {}", config, e))
        .and_then(|s| {
            std::fs::write(CONFIG_FILE_PATH_STR, s)
                .map_err(|e| println!("Could not save to {}: {}", CONFIG_FILE_PATH_STR, e))
        })
}
