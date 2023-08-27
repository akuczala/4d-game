pub mod shape_texture;
pub mod texture_builder;

use std::collections::HashMap;

pub use self::shape_texture::{FaceTexture, FaceTextureBuilder, ShapeTexture, ShapeTextureBuilder};

use super::DrawLine;

use crate::components::{BBox, Convex, HasBBox, Shape, ShapeType, Transform};
use crate::constants::ZERO;
use crate::geometry::affine_transform::AffineTransform;
use crate::geometry::shape::face::FaceBuilder;
use crate::geometry::shape::generic::subface_plane;
use crate::geometry::shape::{Edge, EdgeIndex, FaceIndex, VertIndex};
use crate::geometry::transform::Scaling;
use crate::geometry::{Face, Line, Plane};
use crate::utils::BranchIterator;
use crate::vector::{
    barycenter, random_sphere_point, rotation_matrix, Field, VecIndex, VectorTrait,
};

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

#[derive(Clone, Serialize, Deserialize)]
pub struct UVMap<V, M, U> {
    map: Transform<V, M>, // Used to map from ref space (V) to UV space (U)
    bounding_shape: Shape<U>,
    bbox: BBox<U>,
}
impl<V: VectorTrait> UVMapV<V> {
    pub fn transform_ref_vec_to_uv_vec(&self, point: &V) -> V::SubV {
        self.map.transform_vec(point).project()
    }
    pub fn uv_to_world_transform(
        &self,
        shape_transform: &Transform<V, V::M>,
    ) -> AffineTransform<V, V::M> {
        // TODO: consider caching some of this in the struct
        // We could save calc time by assuming map is ortho affine (no scaling)
        let map_inverse = self.map.inverse();
        AffineTransform::from(*shape_transform).compose(map_inverse)
    }
    pub fn is_point_within_bounds(&self, point: V::SubV) -> bool {
        match self.bounding_shape.shape_type {
            ShapeType::Convex(_) => Convex::point_within(point, ZERO, &self.bounding_shape.faces),
            _ => panic!("Expected convex bounding shape"),
        }
    }
    pub fn bounds(&self) -> impl Iterator<Item = &Plane<V::SubV>> {
        self.bounding_shape.faces.iter().map(|face| face.plane())
    }

    fn draw_lines<'a>(
        &'a self,
        shape_transform: &Transform<V, V::M>,
        lines: &'a [Line<V::SubV>],
    ) -> impl Iterator<Item = Line<V>> + 'a {
        let uv_to_space = self.uv_to_world_transform(shape_transform);
        lines
            .iter()
            .map(move |line| line.map(|p| uv_to_space.transform_vec(&V::unproject(p))))
    }
}
type UVMapV<V> = UVMap<V, <V as VectorTrait>::M, <V as VectorTrait>::SubV>;
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
    // TODO: much of this could be moved to a shapebuilder fn that separates /creates a boundary shape from a face of a given shape
    let verti_map: HashMap<VertIndex, VertIndex> = face
        .vertis
        .iter()
        .enumerate()
        .map(|(new, old)| (*old, new))
        .collect();
    let edgei_map: HashMap<EdgeIndex, EdgeIndex> = face
        .edgeis
        .iter()
        .enumerate()
        .map(|(new, old)| (*old, new))
        .collect();
    let edges: Vec<Edge> = face
        .edgeis
        .iter()
        .map(|ei| shape.edges[*ei].map(|vi| *verti_map.get(&vi).unwrap()))
        .collect();
    let verts: Vec<V::SubV> = face
        .get_verts(&shape.verts)
        .map(|v| map.transform_vec(v).project())
        .collect();
    let face_builder = FaceBuilder::new(&verts, &edges);
    let faces = shape
        .shape_type
        .get_face_subfaces(face_index)
        .map(|subface| {
            let unmapped_edgeis = subface.get_edgeis(&shape.edges, &shape.faces);
            let face_plane = subface_plane(&shape.faces, face_index, &subface)
                .with_transform(map)
                .intersect_proj_plane();
            let edgeis = unmapped_edgeis
                .iter()
                .map(|ei| *edgei_map.get(ei).unwrap())
                .collect();
            face_builder.build(edgeis, face_plane.normal, false)
        })
        .collect();
    let bounding_shape = Shape::new_convex(verts, edges, faces);
    let bbox = bounding_shape.calc_bbox();
    UVMap {
        map,
        bounding_shape,
        bbox,
    }
}
impl<V: VectorTrait> UVMapV<V> {
    pub fn from_frame_texture_mapping(
        ref_shape: &Shape<V>,
        shape_mapping: FrameTextureMapping,
    ) -> Self {
        let origin = shape_mapping.origin(&ref_shape.verts);
        let frame = shape_mapping.frame_verts(&ref_shape.verts);
        let (normed_frame, mut norms): (Vec<V>, Vec<Field>) =
            frame.map(|v| v.normalize_get_norm()).unzip();
        let transform_frame: V::M = normed_frame
            .into_iter()
            .map(|f| (0..V::DIM).map(|i| f.dot(V::one_hot(i))).collect())
            .collect();
        norms.push(1.0);
        let transform_scale = Scaling::Vector(norms.into_iter().collect());
        let map = Transform {
            pos: origin,
            frame: transform_frame,
            scale: transform_scale,
        };
        UVMap {
            map,
            bounding_shape: todo!(),
            bbox: todo!(),
        }
    }
}

pub fn draw_default_on_uv<V: VectorTrait>(
    face_scale: Field,
    uv_map: &UVMapV<V>,
) -> Vec<Line<V::SubV>> {
    let bounds_center = barycenter(&uv_map.bounding_shape.verts);
    let scale_point = |v| V::SubV::linterp(bounds_center, v, face_scale);
    uv_map
        .bounding_shape
        .faces
        .iter()
        .flat_map(|face| {
            face.get_edges(&uv_map.bounding_shape.edges)
                .map(|edge| edge.get_line(&uv_map.bounding_shape.verts).map(scale_point))
        })
        .collect()
}

pub fn draw_fuzz_on_uv<V: VectorTrait>(uv_map: &UVMapV<V>, n: usize) -> Texture<V::SubV> {
    Texture::Lines {
        lines: (0..n * 2)
            .map(|_| uv_map.bbox.random_point())
            .filter(|v| uv_map.is_point_within_bounds(*v))
            .take(n)
            .map(pointlike_line)
            .collect(),
        color: WHITE,
    }
}

// TODO: methods to transform map, likely by taking a transform<V::SubV, V::SubV::M>
// TODO: apply to rendering tiles
// TODO: is the texturemapping frame + origin a special case of UVMap?
// TODO: automagically generate textures based on shape + directives, so we don't need to save them per shape

// Generalize TextureMapping to arbitrary affine transformation?
// Optional additional info to clip out lines that are outside face boundary

// steps for using a UV mapped texture for e.g. fuzz
// have a texture building directive that says
// Default -> MergedWith Fuzz
// this translates to
// 1. Create default texture
// 2. Convert default texture to lines texture
// 2a. Create auto uv map from face
// 2b. draw default texture in UV space
// 3. Create fuzz texture
// 3a. a UV map already exists for this face, so use it
// 3b. draw fuzz lines in UV space
// 4. Merge lines to create new lines texture

// have a shape texture directive that says
// For each face, do (Default -> MergedWith Fuzz -> Color color) for colors in cardinal colors

// Steps for drawing via UV map
// Shape data not needed?
// Compose uv map with shape transform, then apply map to all lines

// TODO: replace texturemapping from serialization with a mapping directive
// e.g. AutoUV, SortLengths
// TODO: above will require separate shape + face builder + drawer structs

#[derive(Clone, Serialize, Deserialize)]
pub struct OldTextureMapping {
    pub frame_vertis: Vec<VertIndex>,
    pub origin_verti: VertIndex,
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub enum TextureMapping<V, M, U> {
    #[default]
    None,
    Frame(FrameTextureMapping),
    UV(UVMap<V, M, U>),
}
pub type TextureMappingV<V> = TextureMapping<V, <V as VectorTrait>::M, <V as VectorTrait>::SubV>;

impl<V: VectorTrait> TextureMappingV<V> {
    fn draw_lines<'a>(
        &'a self,
        shape: &'a Shape<V>,
        shape_transform: &Transform<V, V::M>,
        lines: &'a [Line<V::SubV>],
        color: Color,
    ) -> impl Iterator<Item = DrawLine<V>> + 'a {
        match self {
            Self::None => {
                panic!("Can't draw lines");
                BranchIterator::Option1(std::iter::empty::<DrawLine<V>>())
            }
            Self::Frame(ftm) => BranchIterator::Option2(ftm.draw_lines(shape, lines, color)),
            Self::UV(uv_map) => BranchIterator::Option3(
                uv_map
                    .draw_lines(shape_transform, lines)
                    .map(move |line| DrawLine { line, color }),
            ),
        }
    }
}

// TODO: to replace OldTextureMapping
type FrameTextureMapping = OldTextureMapping;

impl FrameTextureMapping {
    pub fn origin<V: VectorTrait>(&self, shape_verts: &[V]) -> V {
        shape_verts[self.origin_verti]
    }
    // could be represented as a V::SubV::DIM x V::DIM matrix
    pub fn frame_verts<'a, V: VectorTrait>(
        &'a self,
        shape_verts: &'a [V],
    ) -> impl Iterator<Item = V> + 'a {
        self.frame_vertis
            .iter()
            .map(|&vi| shape_verts[vi] - self.origin(shape_verts))
    }
    pub fn draw_lines<'a, V: VectorTrait>(
        &self,
        shape: &'a Shape<V>,
        lines: &'a [Line<V::SubV>],
        color: Color,
    ) -> impl Iterator<Item = DrawLine<V>> + 'a {
        let origin = self.origin(&shape.verts);
        let frame_verts: Vec<V> = self.frame_verts(&shape.verts).collect_vec();
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
        Self {
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
fn test_uv_map_bounds() {
    use crate::constants::ZERO;
    use crate::geometry::{shape::buildshapes::ShapeBuilder, Plane, PointedPlane};
    use crate::tests::{random_rotation_matrix, random_vec};
    use crate::vector::{is_close, is_less_than_or_close, MatrixTrait, Vec4};
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
        // assert that all projected verts are within the boundaries
        for b in uv_map.bounds() {
            assert!(face.get_verts(&shape.verts).all(|v| is_less_than_or_close(
                b.point_signed_distance(uv_map.map.transform_vec(v).project()),
                ZERO
            )));
        }
    }
}

#[test]
fn test_uv_map_shape() {
    use crate::constants::ZERO;
    use crate::geometry::shape::buildshapes::ShapeBuilder;
    use crate::tests::{random_rotation_matrix, random_vec};
    use crate::vector::{is_close, Vec3, Vec4};
    type V = Vec4;
    let random_rotation = random_rotation_matrix::<V>();
    let random_transform: Transform<V, <V as VectorTrait>::M> =
        Transform::new(Some(random_vec()), Some(random_rotation), None);

    let shape: Shape<V> = ShapeBuilder::build_prism(V::DIM, &[2.0, 1.0], &[5, 7])
        .with_transform(random_transform)
        .build();
    for (face_index, face) in shape.faces.iter().enumerate() {
        let uv_map = auto_uv_map_face(&shape, face_index);

        for (p_original, bounding_p) in face
            .get_verts(&shape.verts)
            .zip(uv_map.bounding_shape.verts.iter())
        {
            let p_mapped = uv_map.map.transform_vec(&p_original);
            // assert that the last component of each transformed vec is zero
            assert!(is_close(p_mapped[-1], ZERO));
            // assert that the bounding verts match the projected shape verts
            assert!(Vec3::is_close(p_mapped.project(), *bounding_p));
        }
        for bounding_face in &uv_map.bounding_shape.faces {
            // assert that the vertices of each bounding face lie within the face's plane
            assert!(bounding_face
                .get_verts(&uv_map.bounding_shape.verts)
                .all(|vert| is_close(bounding_face.plane().point_signed_distance(*vert), ZERO)))
        }
    }
}
