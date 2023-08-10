use serde::{Deserialize, Serialize};

use crate::{
    components::Shape,
    constants::CARDINAL_COLORS,
    draw::DrawLine,
    geometry::Face,
    graphics::colors::Color,
    vector::{Field, VectorTrait},
};

use super::{
    draw_default_lines,
    texture_builder::{TextureBuilder, TextureBuilderConfig},
    Texture, TextureMapping,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct ShapeTextureGeneric<T> {
    pub face_textures: Vec<FaceTextureGeneric<T>>, // TODO: replace with a hashmap or vec padded by None to allow defaults?
}
impl<T: Default> ShapeTextureGeneric<T> {
    pub fn new_default(n_faces: usize) -> Self {
        Self {
            face_textures: (0..n_faces).map(|_| Default::default()).collect(),
        }
    }
}

pub type ShapeTexture<U> = ShapeTextureGeneric<Texture<U>>;
pub type ShapeTextureBuilder = ShapeTextureGeneric<TextureBuilder>;

impl<U> ShapeTexture<U> {
    #[allow(dead_code)]
    pub fn with_color(mut self, color: Color) -> Self {
        for face in &mut self.face_textures {
            face.set_color(color);
        }
        self
    }
}
impl ShapeTextureGeneric<TextureBuilder> {
    pub fn with_color(mut self, color: Color) -> Self {
        for face in &mut self.face_textures {
            face.set_color(color);
        }
        self
    }
    pub fn build<U: VectorTrait>(self, config: &TextureBuilderConfig) -> ShapeTexture<U> {
        ShapeTexture {
            face_textures: self
                .face_textures
                .into_iter()
                .map(|ft| ft.build(config))
                .collect(),
        }
    }
}
impl<T: Clone> ShapeTextureGeneric<T> {
    pub fn with_texture(mut self, face_texture: FaceTextureGeneric<T>) -> Self {
        for face in self.face_textures.iter_mut() {
            *face = face_texture.clone();
        }
        self
    }

    #[allow(dead_code)]
    pub fn map_textures<F>(mut self, f: F) -> Self
    where
        F: Fn(FaceTextureGeneric<T>) -> FaceTextureGeneric<T>,
    {
        for face in self.face_textures.iter_mut() {
            take_mut::take(face, &f);
        }
        self
    }
    pub fn zip_textures_with<I, S, F>(mut self, iter: I, f: F) -> Self
    where
        F: Fn(FaceTextureGeneric<T>, S) -> FaceTextureGeneric<T>,
        I: Iterator<Item = S>,
    {
        for (face, item) in self.face_textures.iter_mut().zip(iter) {
            take_mut::take(face, |face| f(face, item));
        }
        self
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct FaceTextureGeneric<T> {
    pub texture: T,
    pub texture_mapping: Option<TextureMapping>,
}
pub type FaceTexture<U> = FaceTextureGeneric<Texture<U>>;
pub type FaceTextureBuilder = FaceTextureGeneric<TextureBuilder>;

impl<T: Default> Default for FaceTextureGeneric<T> {
    fn default() -> Self {
        Self {
            texture: Default::default(),
            texture_mapping: Default::default(),
        }
    }
}
impl<U> FaceTexture<U> {
    pub fn set_color(&mut self, color: Color) {
        take_mut::take(&mut self.texture, |tex| tex.set_color(color));
    }
}
impl FaceTextureGeneric<TextureBuilder> {
    pub fn set_color(&mut self, color: Color) {
        take_mut::take(&mut self.texture, |tex| tex.with_color(color));
    }
    pub fn build<U: VectorTrait>(self, config: &TextureBuilderConfig) -> FaceTexture<U> {
        FaceTexture {
            texture: self.texture.build(config),
            texture_mapping: self.texture_mapping,
        }
    }
}
// this was originally a method of FaceTexture, but I didn't know how to tell rust that U = V::SubV
pub fn draw_face_texture<V: VectorTrait>(
    face_texture: &FaceTexture<V::SubV>,
    face: &Face<V>,
    shape: &Shape<V>,
    face_scales: &[Field],
    visible: bool,
) -> Vec<DrawLine<V>> {
    if !visible {
        return Vec::new();
    }
    match &face_texture.texture {
        Texture::DefaultLines { color } => draw_default_lines(face, shape, *color, face_scales),
        Texture::Lines { lines, color } => face_texture
            .texture_mapping
            .as_ref()
            .unwrap()
            .draw_lines(shape, lines, *color),
        Texture::DrawLines(draw_lines) => face_texture
            .texture_mapping
            .as_ref()
            .unwrap()
            .draw_drawlines(draw_lines),
    }
}

pub fn color_cube_shape_texture<V: VectorTrait>() -> ShapeTextureGeneric<TextureBuilder> {
    ShapeTextureGeneric {
        face_textures: (0..V::DIM * 2)
            .zip(&CARDINAL_COLORS)
            .map(|(_face, &color)| FaceTextureGeneric {
                texture: TextureBuilder::new().with_color(color.set_alpha(0.5)),
                texture_mapping: None,
            })
            .collect(),
    }
}

pub fn fuzzy_color_cube_texture<V: VectorTrait>(
    shape: &Shape<V>,
) -> ShapeTextureGeneric<TextureBuilder> {
    let texture_builder = TextureBuilder::new();
    color_cube_shape_texture::<V>().zip_textures_with(shape.faces.iter(), |face_tex, face| {
        FaceTextureGeneric {
            texture: face_tex
                .texture
                .merged_with(texture_builder.clone().make_fuzz_texture()),
            texture_mapping: Some(TextureMapping::calc_cube_vertis(
                face,
                &shape.verts,
                &shape.edges,
            )),
        }
    })
}

#[allow(dead_code)]
pub fn color_duocylinder<V: VectorTrait>(
    shape_texture: &mut ShapeTexture<V::SubV>,
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
        face.texture = Texture::DefaultLines { color };
    }
}
