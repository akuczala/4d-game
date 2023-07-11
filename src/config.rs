use std::error;

use serde::{Deserialize, Serialize};

use crate::{
    constants::{
        CLIP_SPHERE_RADIUS, CONFIG_FILE_PATH_STR, FACE_SCALE, FOCAL, N_FUZZ_LINES,
        N_HORIZON_FUZZ_LINES, N_SKY_FUZZ_LINES, VIEWPORT_SHAPE,
    },
    draw::ViewportShape,
    vector::Field,
};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ViewConfig {
    pub clip_sphere_radius: Field,
    pub viewport_shape: ViewportShape,
    pub focal: Field,
    pub spin_speed: Field,
}
impl Default for ViewConfig {
    fn default() -> Self {
        Self {
            clip_sphere_radius: CLIP_SPHERE_RADIUS,
            viewport_shape: VIEWPORT_SHAPE,
            focal: FOCAL,
            spin_speed: Default::default(),
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
            face_num: N_FUZZ_LINES,
            sky_num: N_SKY_FUZZ_LINES,
            horizon_num: N_HORIZON_FUZZ_LINES,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Config {
    pub face_scale: Field,
    pub fuzz_lines: FuzzLinesConfig,
    pub view_config: ViewConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            face_scale: FACE_SCALE,
            fuzz_lines: Default::default(),
            view_config: Default::default(),
        }
    }
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
