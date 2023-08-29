use serde::{Deserialize, Serialize};

use crate::{
    components::Shape,
    config::Config,
    geometry::{affine_transform::AffineTransform, shape::FaceIndex},
    graphics::colors::{Color, WHITE},
    vector::{Field, VectorTrait},
};

use super::{
    draw_fuzz_on_uv, make_default_lines_texture, merge_textures,
    shape_texture::TextureMappingDirective, Texture, TextureMappingV, UVMapV,
};

#[derive(Clone, Serialize, Deserialize, Default)]
pub enum TexturePrim {
    Empty,
    #[default]
    Default,
    Tile {
        scales: Vec<Field>,
        n_divisions: Vec<i32>,
    },
    Fuzz,
}
impl TexturePrim {
    fn required_mapping(&self) -> TextureMappingDirective {
        match self {
            Self::Empty => TextureMappingDirective::None,
            Self::Default => TextureMappingDirective::UVDefault,
            Self::Tile { .. } => TextureMappingDirective::Orthogonal,
            Self::Fuzz => TextureMappingDirective::UVDefault,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum TextureBuilderStep {
    WithColor(Color),
    MergedWith(TexturePrim, Vec<TextureBuilderStep>),
}

#[derive(Clone)]
pub struct TextureBuilderConfig {
    n_fuzz_lines: usize,
    face_scale: Field,
}
impl From<&Config> for TextureBuilderConfig {
    fn from(value: &Config) -> Self {
        Self {
            n_fuzz_lines: value.draw.fuzz_lines.face_num,
            face_scale: value.draw.face_scale,
        }
    }
}

type ShapeData<'a, V> = (&'a Shape<V>, &'a Shape<V>, FaceIndex);

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct TextureBuilder {
    start: TexturePrim,
    steps: Vec<TextureBuilderStep>,
}

impl TextureBuilder {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn with_start_texture(mut self, texture_prim: TexturePrim) -> Self {
        self.start = texture_prim;
        self
    }
    pub fn make_tile_texture(self, scales: Vec<Field>, n_divisions: Vec<i32>) -> Self {
        self.with_start_texture(TexturePrim::Tile {
            scales,
            n_divisions,
        })
    }
    pub fn make_fuzz_texture(self) -> Self {
        self.with_start_texture(TexturePrim::Fuzz)
    }

    pub fn merged_with(self, texture_builder: TextureBuilder) -> Self {
        self.with_step(TextureBuilderStep::MergedWith(
            texture_builder.start,
            texture_builder.steps,
        ))
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

    pub fn make_texture_and_map<V: VectorTrait>(
        config: &TextureBuilderConfig,
        start: TexturePrim,
        uv_map: &UVMapV<V>,
    ) -> Texture<V::SubV> {
        {
            match start {
                TexturePrim::Empty => todo!(),
                TexturePrim::Default => {
                    make_default_lines_texture(config.face_scale, uv_map, WHITE)
                }
                TexturePrim::Tile {
                    scales,
                    n_divisions,
                } => Texture::make_tile_texture(&scales, &n_divisions),
                TexturePrim::Fuzz => draw_fuzz_on_uv(uv_map, config.n_fuzz_lines),
            }
        }
    }
    pub fn build<V: VectorTrait>(
        self,
        config: &TextureBuilderConfig,
        ref_shape: &Shape<V>,
        shape: &Shape<V>,
        face_index: FaceIndex,
    ) -> (Texture<V::SubV>, TextureMappingV<V>) {
        let shape_data = (ref_shape, shape, face_index);
        let starting_map = self
            .start
            .required_mapping()
            .build(face_index, ref_shape, shape);
        self.steps.into_iter().fold(
            (
                Self::make_texture_and_map(config, self.start, &starting_map.uv_map),
                starting_map,
            ),
            |(texture, mapping), step| Self::apply_step(config, shape_data, texture, mapping, step),
        )
    }
    fn apply_step<V: VectorTrait>(
        config: &TextureBuilderConfig,
        (ref_shape, shape, face_index): ShapeData<V>,
        texture: Texture<V::SubV>,
        mapping: TextureMappingV<V>,
        step: TextureBuilderStep,
    ) -> (Texture<V::SubV>, TextureMappingV<V>) {
        // TODO: probably much more natural to make this a face texture builder directly
        match step {
            TextureBuilderStep::WithColor(color) => (texture.set_color(color), mapping),
            TextureBuilderStep::MergedWith(start, steps) => {
                let (mut other_texture, other_mapping) = Self::new()
                    .with_start_texture(start)
                    .with_steps(steps)
                    .build::<V>(config, ref_shape, shape, face_index);
                // use UV space from our mapping, rather than other
                //transform lines from other into our map
                let old_to_new_transform = AffineTransform::from(mapping.uv_map.map)
                    .compose(other_mapping.uv_map.map.inverse());
                other_texture.map_lines_in_place(|line| {
                    line.map(|p| {
                        old_to_new_transform
                            .transform_vec(&VectorTrait::unproject(p))
                            .project()
                    })
                });
                (merge_textures::<V>(&texture, &other_texture), mapping)
            }
        }
    }
}
