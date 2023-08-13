pub mod shape_texture;
pub mod texture_builder;

pub use self::shape_texture::{FaceTexture, FaceTextureBuilder, ShapeTexture, ShapeTextureBuilder};

use super::visual_aids::random_sphere_point;
use super::DrawLine;

use crate::components::Shape;
use crate::geometry::shape::{Edge, VertIndex};
use crate::geometry::{Face, Line};
use crate::vector::{Field, VecIndex, VectorTrait};

use crate::graphics::colors::*;

use itertools::Itertools;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub enum Texture<V> {
    DefaultLines { color: Color },
    Lines { lines: Vec<Line<V>>, color: Color },
    DrawLines(Vec<DrawLine<V>>), // I don't remember what this one is for
}
impl<V> Default for Texture<V> {
    fn default() -> Self {
        Self::DefaultLines { color: WHITE }
    }
}
impl<V> Texture<V> {
    pub fn set_color(self, color: Color) -> Self {
        match self {
            Texture::DefaultLines { .. } => Texture::DefaultLines { color },
            Texture::Lines { lines, .. } => Texture::Lines { lines, color },
            Texture::DrawLines(draw_lines) => Texture::DrawLines(
                draw_lines
                    .into_iter()
                    .map(|draw_line| DrawLine {
                        line: draw_line.line,
                        color,
                    })
                    .collect(),
            ),
        }
    }
}
impl<V: VectorTrait> Texture<V> {
    pub fn make_single_tile_texture(color: Color, face_scale: Field) -> Self {
        Texture::make_tile_texture(&[face_scale], &(0..V::DIM).map(|_| 1).collect_vec())
            .set_color(color)
    }
    pub fn make_tile_texture(scales: &[Field], n_divisions: &Vec<i32>) -> Self {
        if V::DIM != n_divisions.len() as VecIndex {
            panic!(
                "make_tile_texture: Expected n_divisions.len()={} but got {}",
                V::DIM,
                n_divisions.len()
            );
        }

        let centers = n_divisions
            .iter()
            .map(|n| (0..*n))
            .multi_cartesian_product()
            .map(|ivec| {
                ivec.iter()
                    .enumerate()
                    .map(|(axis, &i)| {
                        V::one_hot(axis as VecIndex) * ((i as Field) + 0.5)
                            / ((n_divisions[axis]) as Field)
                    })
                    .fold(V::zero(), |v, u| v + u)
            });

        //all this does is convert n_divisions to a vector and divide by 2
        //but since i haven't bothered putting a Vec<Field> -> V function in the vector library
        //i have to do this ridiculous fold
        //see also the computation of the centers
        let corner = n_divisions
            .iter()
            .enumerate()
            .map(|(ax, &n)| V::one_hot(ax as VecIndex) / (n as Field))
            .fold(V::zero(), |v, u| v + u)
            / 2.0;

        //grow edges starting from a line
        let mut tile_lines: Vec<Line<V>> = Vec::new();
        for (i, &n) in n_divisions.iter().enumerate() {
            if i == 0 {
                tile_lines.push(Line(-corner, -corner + V::one_hot(0) / (n as Field)));
            } else {
                let new_dir = V::one_hot(i as VecIndex) / (n as Field);
                let mut new_lines: Vec<Line<V>> = tile_lines
                    .iter()
                    .flat_map(|line| {
                        vec![
                            line.map(|v| v + new_dir),
                            Line(line.0, line.0 + new_dir),
                            Line(line.1, line.1 + new_dir),
                        ]
                    })
                    .collect();

                tile_lines.append(&mut new_lines);
            }
        }

        let lines = centers
            .cartesian_product(scales.iter())
            .flat_map(|(center, &scale)| {
                tile_lines
                    .iter()
                    .map(move |line| line.map(|v| v * scale + center))
            })
            .collect();
        Texture::Lines {
            lines,
            color: DEFAULT_COLOR,
        }
    }
    // this works only for rectangular faces, as is
    pub fn make_fuzz_texture(n: usize) -> Self {
        Texture::Lines {
            lines: (0..n).map(|_| pointlike_line(V::random())).collect(),
            color: DEFAULT_COLOR,
        }
    }
    pub fn merged_with(&self, texture: &Texture<V>, face_scale: Field) -> Texture<V> {
        match (self, texture) {
            // first two cases only work for rectangles
            (Texture::DefaultLines { color: color_1 }, other) => {
                Texture::make_single_tile_texture(*color_1, face_scale)
                    .merged_with(other, face_scale)
            }
            (_, Texture::DefaultLines { color: color_2 }) => self.merged_with(
                &Texture::make_single_tile_texture(*color_2, face_scale),
                face_scale,
            ),
            (
                Texture::Lines {
                    lines: lines_1,
                    color,
                },
                Texture::Lines { lines: lines_2, .. },
            ) => Texture::Lines {
                lines: {
                    let mut lines = lines_1.clone();
                    lines.extend(lines_2.clone());
                    lines
                },
                color: *color,
            },
            _ => panic!("Unsupported texture merge operation"),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TextureMapping {
    pub frame_vertis: Vec<VertIndex>,
    pub origin_verti: VertIndex,
}

impl TextureMapping {
    pub fn draw_lines<V: VectorTrait>(
        &self,
        shape: &Shape<V>,
        lines: &[Line<V::SubV>],
        color: Color,
    ) -> Vec<DrawLine<V>> {
        let origin = shape.verts[self.origin_verti];
        let frame_verts: Vec<V> = self
            .frame_vertis
            .iter()
            .map(|&vi| shape.verts[vi] - origin)
            .collect();
        //this is pretty ridiculous. it just matrix multiplies a matrix of frame_verts (as columns) by each vertex
        //in every line, then adds the origin.
        //TODO: a lot of time is spent doing this calculation
        lines
            .iter()
            .map(|line| {
                line.map(|v| {
                    (0..V::SubV::DIM)
                        .zip(frame_verts.iter())
                        .map(|(i, &f)| f * v[i])
                        .fold(V::zero(), |a, b| a + b)
                        + origin
                })
            })
            .map(|line| DrawLine { line, color })
            .collect()
    }
    pub fn draw_drawlines<V: VectorTrait>(
        &self,
        _draw_lines: &[DrawLine<V::SubV>],
    ) -> Vec<DrawLine<V>> {
        unimplemented!()
        //draw_lines.iter().map(|draw_line| Some(draw_line.clone())).collect()
    }
    //use face edges and reference vertices to determine vertex indices for texture mapping
    //order by side length, in decreasing order
    pub fn calc_cube_vertis<V: VectorTrait>(face: &Face<V>, verts: &[V], edges: &[Edge]) -> Self {
        let face_vertis = &face.vertis;
        let origin_verti = face_vertis[0]; //arbitrary
                                           //get list of vertis connected by an edge to origin verti
        let frame_vertis = face
            .edgeis
            .iter()
            .map(|&ei| &edges[ei])
            .filter_map(|edge| match edge {
                Edge(v1, v2) if *v1 == origin_verti => Some(*v2),
                Edge(v1, v2) if *v2 == origin_verti => Some(*v1),
                _ => None,
            });
        let sorted_frame_vertis: Vec<VertIndex> = frame_vertis
            .map(|vi| (vi, (verts[vi] - verts[origin_verti]).norm()))
            .sorted_by(|a, b| b.1.partial_cmp(&a.1).unwrap())
            .map(|(vi, _v)| vi)
            .collect();
        // for &vi in &sorted_frame_vertis {
        // 	println!("{}",(verts[vi]-verts[origin_verti]).norm() );
        // }
        TextureMapping {
            origin_verti,
            frame_vertis: sorted_frame_vertis,
        }
    }
}

pub fn draw_default_lines<V: VectorTrait>(
    face: &Face<V>,
    shape: &Shape<V>,
    color: Color,
    face_scales: &[Field],
) -> Vec<DrawLine<V>> {
    let mut lines: Vec<DrawLine<V>> = Vec::with_capacity(face.edgeis.len() * face_scales.len());
    for &face_scale in face_scales {
        let scale_point = |v| V::linterp(face.center(), v, face_scale);
        for edgei in &face.edgeis {
            let edge = &shape.edges[*edgei];
            lines.push(DrawLine {
                line: Line(shape.verts[edge.0], shape.verts[edge.1]).map(scale_point),
                color,
            });
        }
    }
    lines
}

pub fn pointlike_line<V: VectorTrait>(pos: V) -> Line<V> {
    Line(pos, pos + random_sphere_point::<V>() * 0.005)
}
