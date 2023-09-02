use serde::{Deserialize, Serialize};

use crate::{
    components::{Shape, Transform},
    config::Config,
    draw::normal_to_color,
    geometry::{affine_transform::AffineTransform, shape::FaceIndex, transform::Scaling},
    graphics::colors::{Color, WHITE},
    vector::{Field, VectorTrait},
};

use super::{
    draw_fuzz_on_uv, make_auto_tile_texture, make_default_lines_texture, make_tile_texture,
    merge_textures, shape_texture::TextureMappingDirective, FaceTexture, Texture, UVMapV,
};

#[derive(Clone, Serialize, Deserialize, Default)]
pub enum TexturePrim {
    Empty,
    #[default]
    Default, // TODO: rename to "simple" or some such thing
    Tile {
        scales: Vec<Field>,
        n_divisions: Vec<i32>,
    },
    AutoTile,
    Fuzz,
}
impl TexturePrim {
    fn required_mapping(&self) -> TextureMappingDirective {
        match self {
            Self::Empty => TextureMappingDirective::None,
            Self::Default => TextureMappingDirective::UVDefault,
            Self::Tile { .. } => TextureMappingDirective::Orthogonal,
            Self::AutoTile => TextureMappingDirective::Orthogonal,
            Self::Fuzz => TextureMappingDirective::UVDefault,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum TextureBuilderStep {
    WithColor(Color),
    WithAlpha(f32),
    //WithTexture(TexturePrim),
    MergedWith(Box<TextureBuilder>),
    ColorByNormal,
}

/// Params relevant to texture building pulled from config
#[derive(Clone)]
pub struct TextureBuilderConfig {
    n_fuzz_lines: usize,
    pub face_scale: Field,
}
impl From<&Config> for TextureBuilderConfig {
    fn from(value: &Config) -> Self {
        Self {
            n_fuzz_lines: value.draw.fuzz_lines.face_num,
            face_scale: value.draw.face_scale,
        }
    }
}

type ShapeData<'a, V, M> = (&'a Shape<V>, &'a Shape<V>, &'a Transform<V, M>, FaceIndex);

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct TextureBuilder {
    start: TexturePrim,
    steps: Vec<TextureBuilderStep>,
}
impl From<TexturePrim> for TextureBuilder {
    fn from(value: TexturePrim) -> Self {
        Self {
            start: value,
            steps: vec![],
        }
    }
}

impl TextureBuilder {
    pub fn new(texture_prim: TexturePrim) -> Self {
        Self {
            start: texture_prim,
            steps: vec![],
        }
    }
    pub fn new_tile_texture(scales: Vec<Field>, n_divisions: Vec<i32>) -> Self {
        Self::new(TexturePrim::Tile {
            scales,
            n_divisions,
        })
    }
    pub fn new_fuzz_texture() -> Self {
        Self::new(TexturePrim::Fuzz)
    }

    pub fn merged_with(self, texture_builder: TextureBuilder) -> Self {
        self.with_step(TextureBuilderStep::MergedWith(Box::new(texture_builder)))
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

    pub fn make_texture<V: VectorTrait>(
        config: &TextureBuilderConfig,
        start: TexturePrim,
        shape_scaling: &Scaling<V>,
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
                } => make_tile_texture(&scales, &n_divisions),
                TexturePrim::AutoTile => {
                    make_auto_tile_texture(config.face_scale, shape_scaling, uv_map)
                }
                TexturePrim::Fuzz => draw_fuzz_on_uv(uv_map, config.n_fuzz_lines),
            }
        }
    }
    pub fn build<V: VectorTrait>(
        self,
        config: &TextureBuilderConfig,
        shape_data @ (ref_shape, shape, shape_transform, face_index): ShapeData<V, V::M>,
    ) -> FaceTexture<V> {
        let starting_map = self
            .start
            .required_mapping()
            .build(face_index, ref_shape, shape);
        self.steps.into_iter().fold(
            FaceTexture {
                texture: Self::make_texture(
                    config,
                    self.start,
                    &shape_transform.scale,
                    &starting_map.uv_map,
                ),
                mapping: starting_map,
            },
            |face_texture, step| Self::apply_step(config, shape_data, face_texture, step),
        )
    }
    fn apply_step<V: VectorTrait>(
        config: &TextureBuilderConfig,
        shape_data @ (ref_shape, _shape, _shape_transform, face_index): ShapeData<V, V::M>,
        face_texture: FaceTexture<V>,
        step: TextureBuilderStep,
    ) -> FaceTexture<V> {
        let texture = face_texture.texture;
        let mapping = face_texture.mapping;
        match step {
            TextureBuilderStep::WithColor(color) => FaceTexture {
                texture: texture.set_color(color),
                mapping,
            },
            TextureBuilderStep::WithAlpha(alpha) => FaceTexture {
                texture: texture.map_color(|color| color.with_alpha(alpha)),
                mapping,
            },
            TextureBuilderStep::ColorByNormal => FaceTexture {
                texture: texture.set_color(normal_to_color(ref_shape.faces[face_index].normal())),
                mapping,
            },
            TextureBuilderStep::MergedWith(boxed_builder) => {
                let mut other_face_texture = (*boxed_builder).build::<V>(config, shape_data);
                // use UV space from our mapping, rather than other
                //transform lines from other into our map
                let old_to_new_transform = AffineTransform::from(mapping.uv_map.map)
                    .compose(other_face_texture.mapping.uv_map.map.inverse());
                other_face_texture.texture.map_lines_in_place(|line| {
                    line.map(|p| {
                        old_to_new_transform
                            .transform_vec(&VectorTrait::unproject(p))
                            .project()
                    })
                });
                FaceTexture {
                    texture: merge_textures::<V::SubV>(&texture, &other_face_texture.texture),
                    mapping,
                }
            }
        }
    }
}
