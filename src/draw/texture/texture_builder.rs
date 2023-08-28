use serde::{Deserialize, Serialize};

use crate::{
    components::Shape,
    config::Config,
    geometry::shape::FaceIndex,
    graphics::colors::Color,
    vector::{Field, VectorTrait},
};

use super::{
    auto_uv_map_face, draw_fuzz_on_uv, merge_textures, shape_texture::TextureMappingDirective,
    FrameTextureMapping, Texture, TextureMapping, TextureMappingV, UVMapV,
};

#[derive(Clone, Serialize, Deserialize)]
pub enum TexturePrim {
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
            Self::Default => TextureMappingDirective::None,
            Self::Tile { .. } => TextureMappingDirective::Orthogonal,
            Self::Fuzz => TextureMappingDirective::UVDefault,
        }
    }
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
    pub fn build<V: VectorTrait>(
        self,
        config: &TextureBuilderConfig,
        ref_shape: &Shape<V>,
        shape: &Shape<V>,
        face_index: FaceIndex,
        mapping: TextureMappingV<V>,
    ) -> (Texture<V::SubV>, TextureMappingV<V>) {
        self.steps
            .into_iter()
            .fold((Default::default(), mapping), |(texture, mapping), step| {
                Self::apply_step(config, ref_shape, shape, face_index, texture, mapping, step)
            })
    }
    fn apply_step<V: VectorTrait>(
        config: &TextureBuilderConfig,
        ref_shape: &Shape<V>,
        shape: &Shape<V>,
        face_index: FaceIndex,
        texture: Texture<V::SubV>,
        mapping: TextureMappingV<V>,
        step: TextureBuilderStep,
    ) -> (Texture<V::SubV>, TextureMappingV<V>) {
        // TODO: probably much more natural to make this a face texture builder directly
        match step {
            TextureBuilderStep::WithTexture(new_texture) => {
                let required_mapping_type = new_texture.required_mapping();
                match new_texture {
                    TexturePrim::Default => (
                        Default::default(),
                        transform_mapping(
                            required_mapping_type,
                            ref_shape,
                            shape,
                            face_index,
                            mapping,
                        ),
                    ),
                    TexturePrim::Tile {
                        scales,
                        n_divisions,
                    } => (
                        Texture::make_tile_texture(&scales, &n_divisions),
                        transform_mapping(
                            required_mapping_type,
                            ref_shape,
                            shape,
                            face_index,
                            mapping,
                        ),
                    ),
                    TexturePrim::Fuzz => {
                        let new_mapping = transform_mapping(
                            required_mapping_type,
                            ref_shape,
                            shape,
                            face_index,
                            mapping,
                        );
                        let uv_map = match &new_mapping {
                            TextureMapping::UV(uv) => uv,
                            _ => unreachable!(),
                        };
                        let texture = draw_fuzz_on_uv(uv_map, config.n_fuzz_lines);
                        (texture, new_mapping)
                    }
                }
            }
            TextureBuilderStep::WithColor(color) => (texture.set_color(color), mapping),
            TextureBuilderStep::MergedWith(steps) => {
                let (new_texture, new_mapping) = Self::new().with_steps(steps).build::<V>(
                    config,
                    ref_shape,
                    shape,
                    face_index,
                    Default::default(),
                );
                let new_uv_map = match (mapping, new_mapping) {
                    (TextureMapping::None, TextureMapping::UV(uv))
                    | (TextureMapping::UV(uv), TextureMapping::None)
                    | (TextureMapping::UV(uv), TextureMapping::UV(_)) => uv,
                    _ => panic!("Unsupported merge"),
                };
                (
                    merge_textures::<V>(&texture, &new_texture, config.face_scale, &new_uv_map),
                    TextureMapping::UV(new_uv_map),
                )
            }
        }
    }
}

/// transforms the texture mapping to the appropriate format, if possible, otherwise overwrites the mapping
fn transform_mapping<V: VectorTrait>(
    required_mapping_type: TextureMappingDirective,
    ref_shape: &Shape<V>,
    shape: &Shape<V>,
    face_index: FaceIndex,
    mapping: TextureMappingV<V>,
) -> TextureMappingV<V> {
    match required_mapping_type {
        TextureMappingDirective::None => TextureMapping::None,
        TextureMappingDirective::Orthogonal => TextureMapping::UV(match mapping {
            TextureMapping::None | TextureMapping::UV(_) => UVMapV::from_frame_texture_mapping(
                ref_shape,
                face_index,
                FrameTextureMapping::calc_cube_vertis(
                    &shape.faces[face_index],
                    &shape.verts,
                    &shape.edges,
                ),
            ),
        }),
        TextureMappingDirective::UVDefault => TextureMapping::UV(match mapping {
            TextureMapping::None => auto_uv_map_face(ref_shape, face_index),
            TextureMapping::UV(uv) => uv,
        }),
    }
}
