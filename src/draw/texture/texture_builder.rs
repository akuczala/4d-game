use serde::{Deserialize, Serialize};

use crate::{
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

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct TextureBuilder {
    steps: Vec<TextureBuilderStep>,
    n_fuzz_lines: usize, //TODO: rm and add to build()
}

impl TextureBuilder {
    pub fn new(n_fuzz_lines: usize) -> Self {
        Self {
            steps: Vec::new(),
            n_fuzz_lines,
        }
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
    pub fn build<U: VectorTrait>(self) -> Texture<U> {
        let n_fuzz_lines = self.n_fuzz_lines;
        self.steps
            .into_iter()
            .fold(Default::default(), |texture, step| {
                Self::apply_step(n_fuzz_lines, texture, step)
            })
    }
    pub fn apply_step<V: VectorTrait>(
        n_fuzz_lines: usize,
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
                TexturePrim::Fuzz => Texture::make_fuzz_texture(n_fuzz_lines),
            },
            TextureBuilderStep::WithColor(color) => texture.set_color(color),
            TextureBuilderStep::MergedWith(steps) => {
                let new_texture = Self::new(n_fuzz_lines).with_steps(steps).build::<V>();
                texture.merged_with(&new_texture)
            }
        }
    }
}
