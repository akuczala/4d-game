use std::iter;

use serde::{Deserialize, Serialize};

use crate::{
    components::{Shape, Transform},
    config::DrawConfig,
    constants::{CARDINAL_COLORS, FUZZY_COLOR_CUBE_LABEL_STR, FUZZY_TILE_LABEL_STR},
    draw::{normal_to_color, DrawLine},
    geometry::shape::FaceIndex,
    graphics::colors::Color,
    utils::{BranchIterator2, ResourceLabel, ValidDimension},
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

// TODO: support face-dependent fns
#[derive(Clone, Serialize, Deserialize, Default)]
pub struct ShapeTextureMap(Vec<TextureBuilderStep>);
impl ShapeTextureMap {
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

#[derive(Clone, Serialize, Deserialize)]
pub enum ShapeTextureBuilder {
    Uniform(TextureBuilder),
    Vec(ShapeTextureBuilderVec),
    FromResource(ShapeTextureLabel, ShapeTextureMap),
}
impl Default for ShapeTextureBuilder {
    fn default() -> Self {
        Self::Uniform(Default::default())
    }
}
impl ShapeTextureBuilder {
    pub fn map(self, f: ShapeTextureMap) -> Self {
        match self {
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
    pub fn with_fuzz(self) -> Self {
        self.map(ShapeTextureMap::single(TextureBuilderStep::MergedWith(
            TextureBuilder::new_fuzz_texture().into()
        )))
    }
    pub fn with_texture(self, texture: TextureBuilder) -> Self {
        Self::Uniform(texture)
    }
    pub fn build<V: VectorTrait>(
        self,
        config: &TextureBuilderConfig,
        draw_config: &DrawConfig, // TODO: do we really need to pass 2 configs
        ref_shape: &Shape<V>,
        shape: &Shape<V>,
    ) -> ShapeTexture<V> {
        match self {
            ShapeTextureBuilder::Uniform(t) => {
                ShapeTextureBuilderVec::new_default(ref_shape.faces.len())
                    .with_texture(t)
                    .build(config, ref_shape, shape)
            }
            ShapeTextureBuilder::Vec(ts) => ts.build(config, ref_shape, shape),
            ShapeTextureBuilder::FromResource(label, map) => (match label {
                label if label == "DefaultOrientationColor".into() => {
                    shape_default_orientation_color_texture(ref_shape)
                }
                label if label == FUZZY_COLOR_CUBE_LABEL_STR.into() => {
                    fuzzy_color_cube_texture::<V>()
                }
                label if label == FUZZY_TILE_LABEL_STR.into() => {
                    build_fuzzy_tile_texture::<V>(draw_config, ref_shape.faces.len())
                }
                _ => panic!("Invalid shape texture label {}", label),
            })
            .map_textures(|tex| map.apply(tex))
            .build(config, ref_shape, shape),
        }
    }
}
#[derive(Clone, Serialize, Deserialize)]
pub struct ShapeTextureBuilderVec {
    face_textures: Vec<TextureBuilder>,
}
// pub enum ShapeTextureBuilder {
//     Uniform{face_texture: FaceTextureBuilder, n_faces: usize},
//     NonUniform(Vec<FaceTextureBuilder>),
// }

// pub struct NonUniformShapeTextureBuilder {
//     face_textures: Vec<FaceTextureBuilder>
// }
impl ShapeTextureBuilderVec {
    // pub fn new_default(n_faces: usize) -> Self {
    //     Self::Uniform { face_texture: Default::default(), n_faces }
    // }
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

pub fn color_cube_shape_texture<V: VectorTrait>() -> ShapeTextureBuilderVec {
    ShapeTextureBuilderVec {
        face_textures: (0..V::DIM * 2)
            .zip(&CARDINAL_COLORS)
            .map(|(_face, &color)| TextureBuilder::default().with_color(color.set_alpha(0.5)))
            .collect(),
    }
}

pub fn fuzzy_color_cube_texture<V: VectorTrait>() -> ShapeTextureBuilderVec {
    color_cube_shape_texture::<V>()
        .map_textures(|texture| texture.merged_with(TextureBuilder::new_fuzz_texture()))
}

pub fn shape_default_orientation_color_texture<V: VectorTrait>(
    ref_shape: &Shape<V>,
) -> ShapeTextureBuilderVec {
    ShapeTextureBuilderVec::new_default(ref_shape.faces.len())
        .zip_textures_with(ref_shape.faces.iter(), |ftb, face| {
            ftb.with_color(normal_to_color(face.normal()))
        })
}

#[allow(dead_code)]
pub fn color_duocylinder<V: VectorTrait>(
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

pub fn build_fuzzy_tile_texture<V: VectorTrait>(
    draw_config: &DrawConfig,
    n_faces: usize,
) -> ShapeTextureBuilderVec {
    ShapeTextureBuilderVec {
        face_textures: (0..n_faces)
            .map(|face_index| {
                TextureBuilder::new_tile_texture(
                        vec![draw_config.face_scale],
                        match V::DIM.into() {
                            ValidDimension::Three => vec![3, 1],
                            ValidDimension::Four => vec![3, 1, 1],
                        },
                    )
                    .merged_with(TextureBuilder::new_fuzz_texture())
                    .with_step(TextureBuilderStep::WithColor(CARDINAL_COLORS[face_index]))
            })
            .collect(),
    }
}

pub type ShapeTextureLabel = ResourceLabel<ShapeTextureBuilder>;
