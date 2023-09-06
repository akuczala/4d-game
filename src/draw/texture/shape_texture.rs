use std::iter;

use serde::{Deserialize, Serialize};

use crate::{
    components::{Shape, Transform},
    constants::{AUTO_TILE_LABEL_STR, COIN_TEXTURE_LABEL_STR, FUZZY_TILE_LABEL_STR},
    draw::DrawLine,
    geometry::shape::FaceIndex,
    graphics::colors::{Color, YELLOW},
    utils::{BranchIterator2, ResourceLabel},
    vector::VectorTrait,
};

use super::{
    auto_uv_map_face,
    texture_builder::{TextureBuilder, TextureBuilderConfig, TextureBuilderStep, TexturePrim},
    FrameTextureMapping, Texture, TextureMapping, TextureMappingV, UVMapV,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct ShapeTextureGeneric<V, M, U> {
    pub face_textures: Vec<Option<FaceTextureGeneric<V, M, U>>>, // TODO: replace with a hashmap or vec padded by None to allow defaults?
}

pub type ShapeTexture<V> = ShapeTextureGeneric<V, <V as VectorTrait>::M, <V as VectorTrait>::SubV>;

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct ShapeTextureMap(Vec<TextureBuilderStep>);
impl ShapeTextureMap {
    fn new() -> Self {
        Self::default()
    }
    fn single(step: TextureBuilderStep) -> Self {
        Self(vec![step])
    }
    fn apply(&self, tex: TextureBuilder) -> TextureBuilder {
        tex.with_steps(self.0.clone())
    }
    fn with_step(mut self, step: TextureBuilderStep) -> Self {
        self.0.push(step);
        self
    }
    fn with_steps(mut self, steps: Self) -> Self {
        for step in steps.0 {
            self.0.push(step);
        }
        self
    }
}
impl From<TextureBuilderStep> for ShapeTextureMap {
    fn from(value: TextureBuilderStep) -> Self {
        Self::single(value)
    }
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub enum ShapeTextureBuilder {
    #[default]
    Default,
    Uniform(TextureBuilder),
    Vec(ShapeTextureBuilderVec),
    FromResource(ShapeTextureLabel, ShapeTextureMap),
}
impl ShapeTextureBuilder {
    pub fn parse_default() -> Self {
        Self::Uniform(
            TextureBuilder::new(Default::default()).merged_with(TextureBuilder::new_fuzz_texture()),
        )
        .map(TextureBuilderStep::ColorByNormal.into())
    }
    pub fn map(self, f: ShapeTextureMap) -> Self {
        match self {
            Self::Default => Self::parse_default().map(f),
            Self::Uniform(t) => Self::Uniform(f.apply(t)),
            Self::Vec(t) => Self::Vec(t.map_textures(|t| f.apply(t))),
            Self::FromResource(label, map) => Self::FromResource(label, map.with_steps(f)),
        }
    }
    pub fn from_resource(label: ShapeTextureLabel) -> Self {
        Self::FromResource(label, Default::default())
    }
    pub fn with_color(self, color: Color) -> Self {
        self.map(ShapeTextureMap::single(TextureBuilderStep::WithColor(
            color,
        )))
    }
    pub fn with_texture(self, texture: TextureBuilder) -> Self {
        Self::Uniform(texture)
    }
    fn finalize(self, n_faces: usize) -> ShapeTextureBuilderVec {
        match self {
            Self::Default => Self::parse_default().finalize(n_faces),
            ShapeTextureBuilder::Uniform(t) => {
                ShapeTextureBuilderVec::new_default(n_faces).with_texture(t)
            }
            ShapeTextureBuilder::Vec(ts) => ts,
            ShapeTextureBuilder::FromResource(label, map) => (match label {
                label if label == AUTO_TILE_LABEL_STR.into() => {
                    ShapeTextureBuilder::Uniform(TextureBuilder::new(TexturePrim::AutoTile))
                }
                label if label == COIN_TEXTURE_LABEL_STR.into() => build_coin_texture(),
                label if label == FUZZY_TILE_LABEL_STR.into() => build_fuzzy_tile_texture(),
                _ => panic!("Invalid shape texture label {}", label),
            })
            .map(map)
            .finalize(n_faces),
        }
    }
    pub fn build<V: VectorTrait>(
        self,
        config: &TextureBuilderConfig,
        ref_shape: &Shape<V>,
        shape: &Shape<V>,
        shape_transform: &Transform<V, V::M>,
    ) -> ShapeTexture<V> {
        self.finalize(ref_shape.faces.len())
            .map_textures(|t| t.with_step(TextureBuilderStep::WithAlpha(config.alpha)))
            .build(config, ref_shape, shape, shape_transform)
    }
}
#[derive(Clone, Serialize, Deserialize)]
pub struct ShapeTextureBuilderVec {
    face_textures: Vec<TextureBuilder>,
}
impl ShapeTextureBuilderVec {
    pub fn new_default(n_faces: usize) -> Self {
        Self {
            face_textures: (0..n_faces).map(|_| Default::default()).collect(),
        }
    }

    pub fn build<V: VectorTrait>(
        self,
        config: &TextureBuilderConfig,
        ref_shape: &Shape<V>,
        shape: &Shape<V>,
        shape_transform: &Transform<V, V::M>,
    ) -> ShapeTexture<V> {
        ShapeTexture {
            face_textures: self
                .face_textures
                .into_iter()
                .enumerate()
                .map(|(face_index, face_texture)| {
                    Some(
                        face_texture.build(config, (ref_shape, shape, shape_transform, face_index)),
                    )
                })
                .collect(),
        }
    }
}
impl ShapeTextureBuilderVec {
    pub fn with_texture(mut self, texture: TextureBuilder) -> Self {
        for face in self.face_textures.iter_mut() {
            *face = texture.clone();
        }
        self
    }

    pub fn map_textures<F>(mut self, f: F) -> Self
    where
        F: Fn(TextureBuilder) -> TextureBuilder,
    {
        for face in self.face_textures.iter_mut() {
            take_mut::take(face, &f);
        }
        self
    }
    #[allow(dead_code)]
    pub fn zip_textures_with<I, S, F>(mut self, iter: I, f: F) -> Self
    where
        F: Fn(TextureBuilder, S) -> TextureBuilder,
        I: Iterator<Item = S>,
    {
        for (face, item) in self.face_textures.iter_mut().zip(iter) {
            take_mut::take(face, |face| f(face, item));
        }
        self
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct FaceTextureGeneric<V, M, U> {
    pub texture: Texture<U>,
    pub mapping: TextureMapping<V, M, U>,
}

pub type FaceTexture<V> = FaceTextureGeneric<V, <V as VectorTrait>::M, <V as VectorTrait>::SubV>;

#[derive(Copy, Clone, Default, Serialize, Deserialize)]
pub enum TextureMappingDirective {
    None,
    Orthogonal,
    #[default]
    UVDefault,
}
impl TextureMappingDirective {
    pub fn build<V: VectorTrait>(
        &self,
        face_index: FaceIndex,
        ref_shape: &Shape<V>,
        shape: &Shape<V>,
    ) -> TextureMappingV<V> {
        match self {
            TextureMappingDirective::Orthogonal => {
                TextureMapping::new(UVMapV::from_frame_texture_mapping(
                    ref_shape,
                    face_index,
                    FrameTextureMapping::calc_cube_vertis(
                        &shape.faces[face_index],
                        &shape.verts,
                        &shape.edges,
                    ),
                ))
            }
            TextureMappingDirective::UVDefault | Self::None => {
                TextureMapping::new(auto_uv_map_face(ref_shape, face_index))
            }
        }
    }
}

pub fn draw_face_texture<'a, V: VectorTrait + 'a>(
    face_texture: &'a FaceTexture<V>,
    shape_transform: &'a Transform<V, V::M>,
    visible: bool,
    override_color: Option<Color>,
) -> impl Iterator<Item = DrawLine<V>> + 'a {
    if !visible {
        return BranchIterator2::Option1(iter::empty());
    }
    match &face_texture.texture {
        Texture::Lines { lines, color } => {
            BranchIterator2::Option2(face_texture.mapping.draw_lines(
                shape_transform,
                lines,
                override_color.unwrap_or(*color),
            ))
        }
        Texture::DrawLines(_) => unimplemented!(),
    }
}

fn build_coin_texture() -> ShapeTextureBuilder {
    ShapeTextureBuilder::Uniform(TexturePrim::Default.into()).with_color(YELLOW)
}

#[allow(dead_code)]
fn color_duocylinder<V: VectorTrait>(
    shape_texture: &mut ShapeTextureBuilderVec,
    m: usize,
    n: usize,
) {
    for (i, face) in itertools::enumerate(shape_texture.face_textures.iter_mut()) {
        let iint = i as i32;
        let color = Color([
            ((iint % (m as i32)) as f32) / (m as f32),
            (i as f32) / ((m + n) as f32),
            1.0,
            1.0,
        ]);
        *face = TextureBuilder::default().with_color(color)
    }
}

fn build_fuzzy_tile_texture() -> ShapeTextureBuilder {
    ShapeTextureBuilder::from_resource(AUTO_TILE_LABEL_STR.into()).map(
        ShapeTextureMap::new()
            .with_step(TextureBuilderStep::MergedWith(
                TextureBuilder::new_fuzz_texture().into(),
            ))
            .with_step(TextureBuilderStep::ColorByNormal),
    )
}

pub type ShapeTextureLabel = ResourceLabel<ShapeTextureBuilder>;
