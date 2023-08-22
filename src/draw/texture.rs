pub mod shape_texture;
pub mod texture_builder;

pub use self::shape_texture::{FaceTexture, FaceTextureBuilder, ShapeTexture, ShapeTextureBuilder};

use super::clipping::boundaries::ConvexBoundarySet;
use super::DrawLine;

use crate::components::{BBox, Shape, Transform};
use crate::geometry::shape::generic::subface_plane;
use crate::geometry::shape::{Edge, FaceIndex, VertIndex};
use crate::geometry::{Face, Line};
use crate::vector::{random_sphere_point, rotation_matrix, Field, VecIndex, VectorTrait};

use crate::graphics::colors::*;

use crate::geometry::Transformable;
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

struct UVMap<V, M, U> {
    map: Transform<V, M>,
    bounds: ConvexBoundarySet<U>,
    bbox: BBox<U>,
}
// We need to be able to generate, for an arbitrary face, a sensible mapping to a D - 1 UV space
fn auto_uv_map_face<V: VectorTrait>(
    shape: &Shape<V>,
    face_index: FaceIndex,
) -> UVMap<V, V::M, V::SubV> {
    // project points of face onto face plane
    // cook up a basis for the plane
    let face = &shape.faces[face_index];
    let basis = rotation_matrix(face.normal(), V::one_hot(-1), None);
    let map = Transform::new(Some(-(basis * face.center())), Some(basis), None);
    let boundaries = shape
        .shape_type
        .get_face_subfaces(face_index)
        .map(|subface| {
            subface_plane(&shape.faces, face_index, &subface)
                .with_transform(map)
                .intersect_proj_plane()
        })
        .collect_vec();
    UVMap {
        map,
        bounds: ConvexBoundarySet(boundaries),
        bbox: BBox::from_verts(
            &face
                .get_verts(&shape.verts)
                .map(|v| map.transform_vec(v).project())
                .collect_vec(),
        ),
    }
}
// TODO: methods to transform map, likely by taking a transform<V::SubV, V::SubV::M>
// TODO: apply to rendering fuzzlines
// TODO: apply to rendering default texture
// TODO: apply to rendering tiles
// TODO: is the texturemapping frame + origin a special case of UVMap?
// TODO: automagically generate textures based on shape + directives, so we don't need to save them per shape

// Generalize TextureMapping to arbitrary affine transformation?
// Optional additional info to clip out lines that are outside face boundary

#[derive(Clone, Serialize, Deserialize)]
pub struct OldTextureMapping {
    pub frame_vertis: Vec<VertIndex>,
    pub origin_verti: VertIndex,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TextureMapping {
    pub frame_vertis: Vec<VertIndex>,
    pub origin_verti: VertIndex,
}

impl TextureMapping {
    pub fn draw_lines<'a, V: VectorTrait>(
        &self,
        shape: &'a Shape<V>,
        lines: &'a [Line<V::SubV>],
        color: Color,
    ) -> impl Iterator<Item = DrawLine<V>> + 'a {
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
            .map(move |line| {
                line.map(|v| {
                    (0..V::SubV::DIM)
                        .zip(frame_verts.iter())
                        .map(|(i, &f)| f * v[i])
                        .fold(V::zero(), |a, b| a + b)
                        + origin
                })
            })
            .map(move |line| DrawLine { line, color })
        //.collect()
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

pub fn draw_default_lines<'a, V: VectorTrait + 'a>(
    face: &'a Face<V>,
    shape: &'a Shape<V>,
    face_scales: &'a [Field],
) -> impl Iterator<Item = Line<V>> + 'a {
    //let mut lines: Vec<DrawLine<V>> = Vec::with_capacity(face.edgeis.len() * face_scales.len());
    face_scales.iter().flat_map(move |face_scale| {
        let scale_point = |v| V::linterp(face.center(), v, *face_scale);
        face.edgeis.iter().map(move |edgei| {
            let edge = &shape.edges[*edgei];
            Line(shape.verts[edge.0], shape.verts[edge.1]).map(scale_point)
        })
    })
}

pub fn pointlike_line<V: VectorTrait>(pos: V) -> Line<V> {
    Line(pos, pos + random_sphere_point::<V>() * 0.005)
}

#[test]
fn test_uv_map() {
    use crate::constants::{TWO_PI, ZERO};
    use crate::geometry::{shape::buildshapes::ShapeBuilder, Plane, PointedPlane};
    use crate::tests::{random_rotation_matrix, random_vec};
    use crate::vector::{is_close, is_less_than_or_close, MatrixTrait, Vec3, Vec4};
    type V = Vec4;
    let random_rotation = random_rotation_matrix::<V>();
    let random_transform: Transform<V, <V as VectorTrait>::M> =
        Transform::new(Some(random_vec()), Some(random_rotation), None);

    let shape: Shape<V> = ShapeBuilder::build_prism(V::DIM, &[2.0, 1.0], &[5, 7])
        .with_transform(random_transform)
        .build();

    let pplane: PointedPlane<V> = shape.faces[0].geometry.clone().into();
    let basis = rotation_matrix(pplane.normal, V::one_hot(-1), None);
    assert!(basis
        .get_rows()
        .iter()
        .take(2)
        .all(|v| is_close(v.dot(pplane.normal), ZERO)));
    let test_point = pplane.point + basis[0] + basis[1];
    assert!(is_close(
        Plane::from(pplane.clone()).point_signed_distance(test_point),
        ZERO
    ));

    for (face_index, face) in shape.faces.iter().enumerate() {
        let uv_map = auto_uv_map_face(&shape, face_index);
        // assert that the zero component of each transformed vec is zero
        for p in face.get_verts(&shape.verts) {
            assert!(is_close(uv_map.map.transform_vec(&p)[-1], ZERO));
        }
        // assert that all projected verts are within the boundaries
        for b in uv_map.bounds.0 {
            assert!(face.get_verts(&shape.verts).all(|v| is_less_than_or_close(
                b.point_signed_distance(uv_map.map.transform_vec(v).project()),
                ZERO
            )));
        }
    }
}
