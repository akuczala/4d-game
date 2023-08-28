use std::iter;

use serde::{Deserialize, Serialize};

use crate::{
    components::{Shape, Transform},
    config::DrawConfig,
    constants::CARDINAL_COLORS,
    draw::DrawLine,
    geometry::{shape::FaceIndex, Face},
    graphics::colors::Color,
    utils::{BranchIterator, ValidDimension},
    vector::{Field, VectorTrait},
};

use super::{
    auto_uv_map_face, draw_default_lines,
    texture_builder::{TextureBuilder, TextureBuilderConfig, TextureBuilderStep, TexturePrim},
    FrameTextureMapping, Texture, TextureMapping, TextureMappingV, UVMapV,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct ShapeTextureGeneric<V, M, U> {
    pub face_textures: Vec<FaceTextureGeneric<V, M, U>>, // TODO: replace with a hashmap or vec padded by None to allow defaults?
}
impl<V: Default, M: Default, U: Default> ShapeTextureGeneric<V, M, U> {
    pub fn new_default(n_faces: usize) -> Self {
        Self {
            face_textures: (0..n_faces).map(|_| Default::default()).collect(),
        }
    }
}

// Examples
// "CoinShapeTexture", "ColorCubeShapeTexture", "FuzzyColorCubeShapeTexture"
// CoinShapeTexture = Uniform(Default >>> Yellow) = DefaultTexture >>> Yellow
// ColorCubeShapeTexture = Map (\color Default >>> color) CardinalColors
// FuzzyColorCubeShapeTexture = Map(\color Default >>> Merge Fuzz >>> color) CardinalColors = Map (\texture texture >>> Merge Fuzz) ColorCubeShapeTexture
// Map: TextureBuilder -> TextureBuilder = TextureBuilder (TextureBuilder is a monoid, if we include identity op(???))
// Additional complication with TextureMapping. We need this to vary with face as well, according to a function

// we may also reduce # stored mappings by fixing an orientation for each face by default (derivable from normal??)
// how to draw fuzz lines / arbitrary textures for an arbitrary convex face?

// Consider using UV mapping?

pub type ShapeTexture<V> = ShapeTextureGeneric<V, <V as VectorTrait>::M, <V as VectorTrait>::SubV>;

// TODO: change to enum, add simplified variants, e.g.
// Uniform(FaceTextureBuilder)
// Mapped(FaceTextureBuilder, Vec<Directives>)
// Inhomogeneous(Vec<FaceTextureBuilder)
#[derive(Clone, Serialize, Deserialize)]
pub struct ShapeTextureBuilder {
    pub face_textures: Vec<FaceTextureBuilder>,
}

impl<V, M, U> ShapeTextureGeneric<V, M, U> {
    #[allow(dead_code)]
    pub fn with_color(mut self, color: Color) -> Self {
        for face in &mut self.face_textures {
            face.set_color(color);
        }
        self
    }
}
impl ShapeTextureBuilder {
    pub fn new_default(n_faces: usize) -> Self {
        Self {
            face_textures: (0..n_faces).map(|_| Default::default()).collect(),
        }
    }
    pub fn with_color(mut self, color: Color) -> Self {
        for face in &mut self.face_textures {
            face.set_color(color);
        }
        self
    }
    pub fn build<V: VectorTrait>(
        self,
        config: &TextureBuilderConfig,
        ref_shape: &Shape<V>,
        shape: &Shape<V>,
    ) -> ShapeTexture<V> {
        ShapeTexture {
            face_textures: self
                .face_textures
                .into_iter()
                .enumerate()
                .map(|(face_index, face_texture)| {
                    face_texture.build(config, face_index, ref_shape, shape)
                })
                .collect(),
        }
    }
}
impl ShapeTextureBuilder {
    pub fn with_texture(mut self, face_texture: FaceTextureBuilder) -> Self {
        for face in self.face_textures.iter_mut() {
            *face = face_texture.clone();
        }
        self
    }

    #[allow(dead_code)]
    pub fn map_textures<F>(mut self, f: F) -> Self
    where
        F: Fn(FaceTextureBuilder) -> FaceTextureBuilder,
    {
        for face in self.face_textures.iter_mut() {
            take_mut::take(face, &f);
        }
        self
    }
    pub fn zip_textures_with<I, S, F>(mut self, iter: I, f: F) -> Self
    where
        F: Fn(FaceTextureBuilder, S) -> FaceTextureBuilder,
        I: Iterator<Item = S>,
    {
        for (face, item) in self.face_textures.iter_mut().zip(iter) {
            take_mut::take(face, |face| f(face, item));
        }
        self
    }
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct FaceTextureGeneric<V, M, U> {
    pub texture: Texture<U>,
    pub texture_mapping: TextureMapping<V, M, U>,
}

// TODO: consider parameterizing this on V instead
pub type FaceTexture<V> = FaceTextureGeneric<V, <V as VectorTrait>::M, <V as VectorTrait>::SubV>;

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct FaceTextureBuilder {
    pub texture: TextureBuilder,
    pub mapping_directive: TextureMappingDirective,
}

impl<V, M, U> FaceTextureGeneric<V, M, U> {
    pub fn set_color(&mut self, color: Color) {
        take_mut::take(&mut self.texture, |tex| tex.set_color(color));
    }
}
impl FaceTextureBuilder {
    pub fn set_color(&mut self, color: Color) {
        take_mut::take(&mut self.texture, |tex| tex.with_color(color));
    }
    pub fn build<V: VectorTrait>(
        self,
        config: &TextureBuilderConfig,
        face_index: FaceIndex,
        ref_shape: &Shape<V>,
        shape: &Shape<V>,
    ) -> FaceTexture<V> {
        let texture_mapping = self.mapping_directive.build(face_index, ref_shape, shape);
        let (texture, texture_mapping) =
            self.texture
                .build(config, ref_shape, shape, face_index, texture_mapping);
        FaceTexture {
            texture,
            texture_mapping,
        }
    }
}

#[derive(Copy, Clone, Default, Serialize, Deserialize)]
pub enum TextureMappingDirective {
    #[default]
    None,
    Orthogonal,
    UVDefault,
}
impl TextureMappingDirective {
    fn build<V: VectorTrait>(
        &self,
        face_index: FaceIndex,
        ref_shape: &Shape<V>,
        shape: &Shape<V>,
    ) -> TextureMappingV<V> {
        match self {
            TextureMappingDirective::None => TextureMapping::None,
            TextureMappingDirective::Orthogonal => {
                TextureMapping::UV(UVMapV::from_frame_texture_mapping(
                    ref_shape,
                    face_index,
                    FrameTextureMapping::calc_cube_vertis(
                        &shape.faces[face_index],
                        &shape.verts,
                        &shape.edges,
                    ),
                ))
            }
            TextureMappingDirective::UVDefault => {
                TextureMapping::UV(auto_uv_map_face(ref_shape, face_index))
            }
        }
    }
}

// this was originally a method of FaceTexture, but I didn't know how to tell rust that U = V::SubV
pub fn draw_face_texture<'a, V: VectorTrait + 'a>(
    face_texture: &'a FaceTexture<V>,
    face: &'a Face<V>,
    shape: &'a Shape<V>,
    shape_transform: &'a Transform<V, V::M>,
    face_scales: &'a [Field],
    visible: bool,
    override_color: Option<Color>,
) -> impl Iterator<Item = DrawLine<V>> + 'a {
    if !visible {
        return BranchIterator::Option3(iter::empty());
    }
    match &face_texture.texture {
        Texture::DefaultLines { color } => {
            BranchIterator::Option1(draw_default_lines(face, shape, face_scales).map(|line| {
                DrawLine {
                    line,
                    color: *color,
                }
            }))
        }
        Texture::Lines { lines, color } => {
            BranchIterator::Option2(face_texture.texture_mapping.draw_lines(
                shape_transform,
                lines,
                override_color.unwrap_or(*color),
            ))
        }
        Texture::DrawLines(_) => unimplemented!(),
    }
}

pub fn color_cube_shape_texture<V: VectorTrait>() -> ShapeTextureBuilder {
    ShapeTextureBuilder {
        face_textures: (0..V::DIM * 2)
            .zip(&CARDINAL_COLORS)
            .map(|(_face, &color)| FaceTextureBuilder {
                texture: TextureBuilder::new().with_color(color.set_alpha(0.5)),
                mapping_directive: TextureMappingDirective::None,
            })
            .collect(),
    }
}

pub fn fuzzy_color_cube_texture<V: VectorTrait>() -> ShapeTextureBuilder {
    let texture_builder = TextureBuilder::new();
    color_cube_shape_texture::<V>().map_textures(|face_tex| FaceTextureBuilder {
        texture: face_tex
            .texture
            .merged_with(texture_builder.clone().make_fuzz_texture()),
        mapping_directive: TextureMappingDirective::Orthogonal,
    })
}

#[allow(dead_code)]
pub fn color_duocylinder<V: VectorTrait>(shape_texture: &mut ShapeTexture<V>, m: usize, n: usize) {
    for (i, face) in itertools::enumerate(shape_texture.face_textures.iter_mut()) {
        let iint = i as i32;
        let color = Color([
            ((iint % (m as i32)) as f32) / (m as f32),
            (i as f32) / ((m + n) as f32),
            1.0,
            1.0,
        ]);
        face.texture = Texture::DefaultLines { color };
    }
}

pub fn build_fuzzy_tile_texture<V: VectorTrait>(
    draw_config: &DrawConfig,
    n_faces: usize,
) -> ShapeTextureBuilder {
    ShapeTextureBuilder {
        face_textures: (0..n_faces)
            .map(|face_index| FaceTextureBuilder {
                texture: TextureBuilder::new()
                    .with_step(TextureBuilderStep::WithTexture(TexturePrim::Tile {
                        scales: vec![draw_config.face_scale],
                        n_divisions: match V::DIM.into() {
                            ValidDimension::Three => vec![3, 1],
                            ValidDimension::Four => vec![3, 1, 1],
                        },
                    }))
                    // TODO: debug; rv
                    // .with_step(TextureBuilderStep::MergedWith(vec![
                    //     TextureBuilderStep::WithTexture(TexturePrim::Fuzz),
                    // ]))
                    .with_step(TextureBuilderStep::WithColor(CARDINAL_COLORS[face_index])),
                mapping_directive: TextureMappingDirective::Orthogonal,
            })
            .collect(),
    }
}
