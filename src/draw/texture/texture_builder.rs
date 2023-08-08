use serde::{Deserialize, Serialize};

use crate::{
    config::Config,
    graphics::colors::Color,
    vector::{Field, VectorTrait},
};

use super::Texture;

#[derive(Clone, Serialize, Deserialize)]
pub enum TexturePrim {
    Default,
    Tile {
        scales: Vec<Field>,
        n_divisions: Vec<i32>,
    },
    Fuzz,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum TextureBuilderStep {
    WithTexture(TexturePrim),
    WithColor(Color),
    MergedWith(Vec<TextureBuilderStep>),
}

#[derive(Clone)]
pub struct TextureBuilderConfig {
    n_fuzz_lines: usize,
}
impl From<&Config> for TextureBuilderConfig {
    fn from(value: &Config) -> Self {
        Self {
            n_fuzz_lines: value.fuzz_lines.face_num,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct TextureBuilder {
    steps: Vec<TextureBuilderStep>,
}

impl TextureBuilder {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn make_tile_texture(self, scales: Vec<Field>, n_divisions: Vec<i32>) -> Self {
        self.with_step(TextureBuilderStep::WithTexture(TexturePrim::Tile {
            scales,
            n_divisions,
        }))
    }
    pub fn merged_with(self, texture_builder: TextureBuilder) -> Self {
        self.with_step(TextureBuilderStep::MergedWith(texture_builder.steps))
    }
    pub fn make_fuzz_texture(self) -> Self {
        self.with_step(TextureBuilderStep::WithTexture(TexturePrim::Fuzz))
    }
    pub fn with_step(mut self, step: TextureBuilderStep) -> Self {
        self.steps.push(step);
        self
    }
    pub fn with_steps(mut self, mut steps: Vec<TextureBuilderStep>) -> Self {
        self.steps.append(&mut steps);
        self
    }
    pub fn with_color(self, color: Color) -> Self {
        self.with_step(TextureBuilderStep::WithColor(color))
    }
    pub fn build<U: VectorTrait>(self, config: &TextureBuilderConfig) -> Texture<U> {
        self.steps
            .into_iter()
            .fold(Default::default(), |texture, step| {
                Self::apply_step(config, texture, step)
            })
    }
    pub fn apply_step<V: VectorTrait>(
        config: &TextureBuilderConfig,
        texture: Texture<V>,
        step: TextureBuilderStep,
    ) -> Texture<V> {
        match step {
            TextureBuilderStep::WithTexture(new_texture) => match new_texture {
                TexturePrim::Default => Default::default(),
                TexturePrim::Tile {
                    scales,
                    n_divisions,
                } => Texture::make_tile_texture(&scales, &n_divisions),
                TexturePrim::Fuzz => Texture::make_fuzz_texture(config.n_fuzz_lines),
            },
            TextureBuilderStep::WithColor(color) => texture.set_color(color),
            TextureBuilderStep::MergedWith(steps) => {
                let new_texture = Self::new().with_steps(steps).build::<V>(config);
                texture.merged_with(&new_texture)
            }
        }
    }
}
