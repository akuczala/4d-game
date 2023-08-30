use std::iter;

use serde::{Deserialize, Serialize};

use crate::{
    components::{Shape, Transform},
    config::DrawConfig,
    constants::CARDINAL_COLORS,
    draw::{normal_to_color, DrawLine},
    geometry::shape::FaceIndex,
    graphics::colors::Color,
    utils::{BranchIterator2, ValidDimension},
    vector::VectorTrait,
};

use super::{
    auto_uv_map_face,
    texture_builder::{TextureBuilder, TextureBuilderConfig, TextureBuilderStep},
    FrameTextureMapping, Texture, TextureMapping, TextureMappingV, UVMapV,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct ShapeTextureGeneric<V, M, U> {
    pub face_textures: Vec<Option<FaceTextureGeneric<V, M, U>>>, // TODO: replace with a hashmap or vec padded by None to allow defaults?
}

// Examples
// "CoinShapeTexture", "ColorCubeShapeTexture", "FuzzyColorCubeShapeTexture"
// CoinShapeTexture = Uniform(Default >>> Yellow) = DefaultTexture >>> Yellow
// ColorCubeShapeTexture = Map (\color Default >>> color) CardinalColors
// FuzzyColorCubeShapeTexture = Map(\color Default >>> Merge Fuzz >>> color) CardinalColors = Map (\texture texture >>> Merge Fuzz) ColorCubeShapeTexture
// Map: TextureBuilder -> TextureBuilder = TextureBuilder (TextureBuilder is a monoid, if we include identity op(???))
// Additional complication with TextureMapping. We need this to vary with face as well, according to a function

// we may also reduce # stored mappings by fixing an orientation for each face by default (derivable from normal??)

pub type ShapeTexture<V> = ShapeTextureGeneric<V, <V as VectorTrait>::M, <V as VectorTrait>::SubV>;

// TODO: change to enum, add simplified variants, e.g.
// Uniform(FaceTextureBuilder)
// Mapped(FaceTextureBuilder, Vec<Directives>)
// Inhomogeneous(Vec<FaceTextureBuilder)
#[derive(Clone, Serialize, Deserialize)]
pub struct ShapeTextureBuilder {
    face_textures: Vec<TextureBuilder>,
}
// pub enum ShapeTextureBuilder {
//     Uniform{face_texture: FaceTextureBuilder, n_faces: usize},
//     NonUniform(Vec<FaceTextureBuilder>),
// }

// pub struct NonUniformShapeTextureBuilder {
//     face_textures: Vec<FaceTextureBuilder>
// }
impl ShapeTextureBuilder {
    // pub fn new_default(n_faces: usize) -> Self {
    //     Self::Uniform { face_texture: Default::default(), n_faces }
    // }
    pub fn new_default(n_faces: usize) -> Self {
        Self {
            face_textures: (0..n_faces).map(|_| Default::default()).collect(),
        }
    }

    pub fn with_color(mut self, color: Color) -> Self {
        for face in &mut self.face_textures {
            take_mut::take(face, |texture| texture.with_color(color));
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
                    Some(face_texture.build(config, ref_shape, shape, face_index))
                })
                .collect(),
        }
    }
}
impl ShapeTextureBuilder {
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

    pub fn with_fuzz(self) -> Self {
        self.map_textures(|texture| texture.merged_with(TextureBuilder::new().make_fuzz_texture()))
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct FaceTextureGeneric<V, M, U> {
    pub texture: Texture<U>,
    pub texture_mapping: TextureMapping<V, M, U>,
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

// this was originally a method of FaceTexture, but I didn't know how to tell rust that U = V::SubV
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
            BranchIterator2::Option2(face_texture.texture_mapping.draw_lines(
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
            .map(|(_face, &color)| TextureBuilder::new().with_color(color.set_alpha(0.5)))
            .collect(),
    }
}

pub fn fuzzy_color_cube_texture<V: VectorTrait>() -> ShapeTextureBuilder {
    color_cube_shape_texture::<V>()
        .map_textures(|texture| texture.merged_with(TextureBuilder::new().make_fuzz_texture()))
}

pub fn shape_default_orientation_color_texture<V: VectorTrait>(
    ref_shape: &Shape<V>,
) -> ShapeTextureBuilder {
    ShapeTextureBuilder::new_default(ref_shape.faces.len())
        .zip_textures_with(ref_shape.faces.iter(), |ftb, face| {
            ftb.with_color(normal_to_color(face.normal()))
        })
}

#[allow(dead_code)]
pub fn color_duocylinder<V: VectorTrait>(
    shape_texture: &mut ShapeTextureBuilder,
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
        *face = TextureBuilder::new().with_color(color)
    }
}

pub fn build_fuzzy_tile_texture<V: VectorTrait>(
    draw_config: &DrawConfig,
    n_faces: usize,
) -> ShapeTextureBuilder {
    ShapeTextureBuilder {
        face_textures: (0..n_faces)
            .map(|face_index| {
                TextureBuilder::new()
                    .make_tile_texture(
                        vec![draw_config.face_scale],
                        match V::DIM.into() {
                            ValidDimension::Three => vec![3, 1],
                            ValidDimension::Four => vec![3, 1, 1],
                        },
                    )
                    .merged_with(TextureBuilder::new().make_fuzz_texture())
                    .with_step(TextureBuilderStep::WithColor(CARDINAL_COLORS[face_index]))
            })
            .collect(),
    }
}
