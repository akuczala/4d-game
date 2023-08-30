pub mod shape_texture;
pub mod texture_builder;

use std::collections::HashMap;

pub use self::shape_texture::{FaceTexture, ShapeTexture, ShapeTextureBuilder};

use super::DrawLine;

use crate::components::{BBox, Convex, HasBBox, Shape, ShapeType, Transform};
use crate::constants::{HALF, ZERO};
use crate::geometry::affine_transform::AffineTransform;
use crate::geometry::shape::face::FaceBuilder;
use crate::geometry::shape::generic::subface_plane;
use crate::geometry::shape::{Edge, EdgeIndex, FaceIndex, VertIndex};
use crate::geometry::transform::Scaling;
use crate::geometry::{Face, Line, Plane};

use crate::vector::{
    barycenter, is_orthonormal_basis, random_sphere_point, rotation_matrix, Field, VecIndex,
    VectorTrait,
};

use crate::graphics::colors::*;

use crate::geometry::Transformable;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub enum Texture<V> {
    Lines { lines: Vec<Line<V>>, color: Color },
    DrawLines(Vec<DrawLine<V>>),
}
impl<V> Default for Texture<V> {
    fn default() -> Self {
        Self::Lines {
            lines: Vec::new(),
            color: WHITE,
        }
    }
}
impl<V> Texture<V> {
    pub fn set_color(self, color: Color) -> Self {
        match self {
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
    // TODO: create in-place version of this if allocation is a problem
    pub fn map_lines<W, F: Fn(Line<V>) -> Line<W>>(self, f: F) -> Texture<W> {
        match self {
            Texture::Lines { lines, color } => Texture::Lines {
                lines: lines.into_iter().map(f).collect(),
                color,
            },
            Texture::DrawLines(draw_lines) => Texture::DrawLines(
                draw_lines
                    .into_iter()
                    .map(move |dl| dl.map_line(&f))
                    .collect(),
            ),
        }
    }
}
// TODO: look into using mem::take here
impl<V: Copy> Texture<V> {
    pub fn map_lines_in_place<F: Fn(Line<V>) -> Line<V>>(&mut self, f: F) {
        match self {
            Texture::Lines { lines, .. } => {
                for line in lines {
                    *line = f((*line).clone())
                }
            }
            Texture::DrawLines(draw_lines) => {
                for draw_line in draw_lines {
                    *draw_line = (draw_line.clone()).map_line(&f)
                }
            }
        }
    }
}

impl<V: VectorTrait> Texture<V> {
    pub fn make_tile_texture(scales: &[Field], n_divisions: &[i32]) -> Self {
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
                    .map(|(axis, &i)| ((i as Field) + HALF) / ((n_divisions[axis]) as Field))
                    .collect::<V>()
            });

        let corner: V = n_divisions
            .iter()
            .map(|n| 1.0 / (*n as Field))
            .collect::<V>()
            * HALF;

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
}
pub fn merge_textures<V: VectorTrait>(
    texture_1: &Texture<V::SubV>,
    texture: &Texture<V::SubV>,
) -> Texture<V::SubV> {
    match (texture_1, texture) {
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
        }, // TODO: check if the colors are the same; if not, create a drawlines texture with both
        _ => panic!("Unsupported texture merge operation"),
    }
}
#[derive(Clone, Serialize, Deserialize)]
pub struct UVMap<V, M, U> {
    map: Transform<V, M>, // Used to map from ref space (V) to UV space (U)
    bounding_shape: Shape<U>,
    bbox: BBox<U>,
}
impl<V: VectorTrait> UVMapV<V> {
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

    fn clip_point(&self, point: V::SubV) -> Option<V::SubV> {
        self.is_point_within_bounds(point).then_some(point)
    }
}
type UVMapV<V> = UVMap<V, <V as VectorTrait>::M, <V as VectorTrait>::SubV>;

fn shape_into_uv_bounding_shape<V: VectorTrait>(
    shape: &Shape<V>,
    face_index: FaceIndex,
    map: &Transform<V, V::M>,
) -> Shape<V::SubV> {
    let face = &shape.faces[face_index];
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
                .with_transform(*map)
                .intersect_proj_plane();
            let edgeis = unmapped_edgeis
                .iter()
                .map(|ei| *edgei_map.get(ei).unwrap())
                .collect();
            face_builder.build(edgeis, face_plane.normal, false)
        })
        .collect();
    Shape::new_convex(verts, edges, faces)
}

/// Generates, for an arbitrary face, a mapping into a D - 1 UV space
fn auto_uv_map_face<V: VectorTrait>(
    ref_shape: &Shape<V>,
    face_index: FaceIndex,
) -> UVMap<V, V::M, V::SubV> {
    // project points of face onto face plane
    // cook up a basis for the plane
    let face = &ref_shape.faces[face_index];
    let basis = rotation_matrix(face.normal(), V::one_hot(-1), None);
    let map = Transform::new(Some(-(basis * face.center())), Some(basis), None);
    let bounding_shape = shape_into_uv_bounding_shape(ref_shape, face_index, &map);
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
        face_index: FaceIndex,
        frame_mapping: FrameTextureMapping,
    ) -> Self {
        let origin = frame_mapping.origin(&ref_shape.verts);
        let frame = frame_mapping.frame_vecs(&ref_shape.verts);
        let (mut normed_frame, mut norms): (Vec<V>, Vec<Field>) =
            frame.map(|v| v.normalize_get_norm()).unzip();
        normed_frame.push(ref_shape.faces[face_index].normal()); // Normal should be perp to frame
        norms.push(1.0); // Doesn't matter
        assert!(is_orthonormal_basis(&normed_frame));
        assert_eq!(normed_frame.len(), V::DIM as usize);
        let transform_frame: V::M = normed_frame
            .into_iter()
            .map(|f| (0..V::DIM).map(|i| f.dot(V::one_hot(i))).collect())
            .collect();

        let transform_scale = Scaling::Vector(norms.into_iter().collect());
        let map = Transform {
            pos: -(transform_frame * transform_scale.scale_vec(origin)),
            //pos: origin,
            frame: transform_frame,
            scale: transform_scale,
        };
        let bounding_shape = shape_into_uv_bounding_shape(ref_shape, face_index, &map);
        let bbox = bounding_shape.calc_bbox();
        UVMap {
            map,
            bounding_shape,
            bbox,
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

pub fn make_default_lines_texture<V: VectorTrait>(
    face_scale: Field,
    uv_map: &UVMapV<V>,
    color: Color,
) -> Texture<V::SubV> {
    Texture::Lines {
        lines: draw_default_on_uv(face_scale, uv_map),
        color,
    }
}

pub fn draw_fuzz_on_uv<V: VectorTrait>(uv_map: &UVMapV<V>, n: usize) -> Texture<V::SubV> {
    Texture::Lines {
        lines: (0..n * 100)
            .map(|_| uv_map.bbox.random_point())
            .filter_map(|p| uv_map.clip_point(p))
            .take(n)
            .map(pointlike_line)
            .collect(),
        color: WHITE,
    }
}

// TODO: automagically generate textures based on shape + directives, so we don't need to save them per shape

// Generalize TextureMapping to arbitrary affine transformation?
// Optional additional info to clip out lines that are outside face boundary

// have a shape texture directive that says
// For each face, do (Default -> MergedWith Fuzz -> Color color) for colors in cardinal colors

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct FrameTextureMapping {
    pub frame_vertis: Vec<VertIndex>,
    pub origin_verti: VertIndex,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TextureMapping<V, M, U> {
    uv_map: UVMap<V, M, U>,
}
impl<V, M, U> TextureMapping<V, M, U> {
    pub fn new(uv_map: UVMap<V, M, U>) -> Self {
        Self { uv_map }
    }
}

pub type TextureMappingV<V> = TextureMapping<V, <V as VectorTrait>::M, <V as VectorTrait>::SubV>;

impl<V: VectorTrait> TextureMappingV<V> {
    fn draw_lines<'a>(
        &'a self,
        shape_transform: &Transform<V, V::M>,
        lines: &'a [Line<V::SubV>],
        color: Color,
    ) -> impl Iterator<Item = DrawLine<V>> + 'a {
        self.uv_map
            .draw_lines(shape_transform, lines)
            .map(move |line| DrawLine { line, color })
    }
}

// TODO: to rm OldTextureMapping
type OldTextureMapping = FrameTextureMapping;

impl FrameTextureMapping {
    pub fn origin<V: VectorTrait>(&self, shape_verts: &[V]) -> V {
        shape_verts[self.origin_verti]
    }
    // could be represented as a V::SubV::DIM x V::DIM matrix
    pub fn frame_vecs<'a, V: VectorTrait>(
        &'a self,
        shape_verts: &'a [V],
    ) -> impl Iterator<Item = V> + 'a {
        self.frame_vertis
            .iter()
            .map(|&vi| shape_verts[vi] - self.origin(shape_verts))
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

pub fn pointlike_line<V: VectorTrait>(pos: V) -> Line<V> {
    Line(pos, pos + random_sphere_point::<V>() * 0.005)
}

#[test]
fn test_uv_map_bounds() {
    use crate::constants::{
        INVERTED_CUBE_LABEL_STR, ONE_SIDED_FACE_LABEL_STR, OPEN_CUBE_LABEL_STR, ZERO,
    };
    use crate::geometry::shape::build_shape_library;
    use crate::geometry::{shape::buildshapes::ShapeBuilder, Plane, PointedPlane};
    use crate::tests::{random_rotation_matrix, random_vec};
    use crate::vector::{is_less_than_or_close, IsClose, MatrixTrait, Vec4};
    type V = Vec4;

    fn assert_on_bounds<V: VectorTrait>(shape: &Shape<V>) {
        let pplane: PointedPlane<V> = shape.faces[0].geometry.clone().into();
        let basis = rotation_matrix(pplane.normal, V::one_hot(-1), None);
        assert!(basis
            .get_rows()
            .iter()
            .take(2)
            .all(|v| IsClose::is_close(v.dot(pplane.normal), ZERO)));
        let test_point = pplane.point + basis[0] + basis[1];
        assert!(IsClose::is_close(
            Plane::from(pplane.clone()).point_signed_distance(test_point),
            ZERO
        ));

        for (face_index, face) in shape.faces.iter().enumerate() {
            let uv_map = auto_uv_map_face(&shape, face_index);
            //println!("face verts {:?}", face.get_verts(&shape.verts).collect_vec());
            //println!("subface planes {:?}", shape.shape_type.get_face_subfaces(face_index).map(|sf| subface_plane(&shape.faces, face_index, &sf)).collect_vec());
            // assert that all projected verts are within the boundaries
            for b in uv_map.bounds() {
                //println!("plane: {:?}", b);
                assert!(face.get_verts(&shape.verts).all(|v| is_less_than_or_close(
                    b.point_signed_distance(uv_map.map.transform_vec(v).project()),
                    ZERO
                )));
            }
        }
    }
    let random_rotation = random_rotation_matrix::<V>();
    let random_transform: Transform<V, <V as VectorTrait>::M> =
        Transform::new(Some(random_vec()), Some(random_rotation), None);
    //let random_transform = Transform::identity();

    let shape: Shape<V> = ShapeBuilder::build_prism(V::DIM, &[2.0, 1.0], &[5, 7])
        .with_transform(random_transform)
        .build();
    assert_on_bounds(&shape);

    let ref_shapes = build_shape_library::<V>();
    for label in [
        ONE_SIDED_FACE_LABEL_STR,
        INVERTED_CUBE_LABEL_STR,
        OPEN_CUBE_LABEL_STR,
    ] {
        let mut shape = ref_shapes.get_unwrap(&label.into()).clone();
        shape.modify(&random_transform);
        assert_on_bounds(&shape);
    }
}

#[test]
fn test_uv_map_shape() {
    use crate::constants::ZERO;
    use crate::geometry::shape::buildshapes::ShapeBuilder;
    use crate::tests::{random_rotation_matrix, random_vec};
    use crate::vector::{IsClose, Vec3, Vec4};
    type V = Vec4;
    let random_rotation = random_rotation_matrix::<V>();
    let random_transform: Transform<V, <V as VectorTrait>::M> =
        Transform::new(Some(random_vec()), Some(random_rotation), None);

    let shape: Shape<V> = ShapeBuilder::build_prism(V::DIM, &[2.0, 1.0], &[5, 7])
        .with_transform(random_transform)
        .build();

    // TODO: test on other shapes
    for (face_index, face) in shape.faces.iter().enumerate() {
        let uv_map = auto_uv_map_face(&shape, face_index);

        // assert that the face center is mapped to the origin
        assert!(IsClose::is_close(
            uv_map.map.transform_vec(&face.center()).project(),
            Vec3::zero()
        ));

        for (p_original, bounding_p) in face
            .get_verts(&shape.verts)
            .zip(uv_map.bounding_shape.verts.iter())
        {
            let p_mapped = uv_map.map.transform_vec(&p_original);
            // assert that the last component of each transformed vec is zero
            assert!(IsClose::is_close(p_mapped[-1], ZERO));
            // assert that the bounding verts match the projected shape verts
            assert!(Vec3::is_close(p_mapped.project(), *bounding_p));
        }
        for bounding_face in &uv_map.bounding_shape.faces {
            // assert that the vertices of each bounding face lie within the face's plane
            assert!(bounding_face
                .get_verts(&uv_map.bounding_shape.verts)
                .all(|vert| IsClose::is_close(
                    bounding_face.plane().point_signed_distance(*vert),
                    ZERO
                )))
        }
    }
}

#[test]
fn test_frame_to_uv() {
    use crate::geometry::shape::buildshapes::ShapeBuilder;
    use crate::tests::random_transform;
    use crate::vector::{IsClose, Vec4};
    type V = Vec4;
    type SubV = <V as VectorTrait>::SubV;
    let random_transform: Transform<V, <V as VectorTrait>::M> = random_transform();

    let ref_shape: Shape<V> = ShapeBuilder::build_cube(2.0)
        .with_transform(random_transform)
        .build();

    let mut shape = ref_shape.clone();
    shape.modify(&Transform::identity().with_scale(Scaling::Vector(V::new(4.0, 3.0, 2.0, 1.0))));

    for (face_index, face) in shape.faces.iter().enumerate() {
        let frame_map = FrameTextureMapping::calc_cube_vertis(face, &shape.verts, &shape.edges);
        let uv_map = UVMapV::from_frame_texture_mapping(&ref_shape, face_index, frame_map.clone());

        // TODO: add is_close trait for float comparisons
        // assert that the frame origin is mapped to zero
        let origin_pt = frame_map.origin(&ref_shape.verts);
        assert!(IsClose::is_close(
            uv_map.map.transform_vec(&origin_pt).project(),
            SubV::zero()
        ));

        let frame_vecs = frame_map.frame_vecs(&ref_shape.verts).collect_vec();
        let mapped_frame_vecs: Vec<_> = frame_vecs
            .iter()
            .map(|p| (uv_map.map.frame * (*p)).project())
            .collect();
        //let frame_verts: Vec<_> = frame_vecs.iter().map(|v| origin_pt + *v).collect();
        //let mapped_frame_verts: Vec<_> = frame_verts.iter().map(|p| uv_map.transform_ref_vec_to_uv_vec(p)).collect();

        // check that the mapped frame vecs are parallel to the expected axis
        // would have liked to used the mapped verts but i am confused about composition
        assert!(mapped_frame_vecs
            .iter()
            .enumerate()
            .all(|(i, fvec)| IsClose::is_close(
                SubV::one_hot(i as VecIndex).dot(*fvec),
                fvec.norm()
            )));

        // confused about the scaling aspect, but works well enough for now
    }

    //TODO: finish
}

#[test]
fn test_fuzz_on_uv() {
    use crate::geometry::shape::buildshapes::ShapeBuilder;
    use crate::tests::{random_rotation_matrix, random_vec};
    use crate::vector::Vec3;
    type V = Vec3;
    let random_rotation = random_rotation_matrix::<V>();
    let random_transform: Transform<V, <V as VectorTrait>::M> =
        Transform::new(Some(random_vec()), Some(random_rotation), None);

    let ref_shape: Shape<V> = ShapeBuilder::build_cube(2.0)
        .with_transform(random_transform)
        .build();

    for (face_index, _face) in ref_shape.faces.iter().enumerate() {
        let uv_map = auto_uv_map_face(&ref_shape, face_index);
        let n = 100;
        let texture = draw_fuzz_on_uv(&uv_map, n);
        match texture {
            Texture::Lines { lines, color: _ } => {
                assert_eq!(lines.len(), n)
            }
            _ => panic!("Expected lines texture"),
        }
    }

    let ref_shape: Shape<V> = ShapeBuilder::build_prism(V::DIM, &[2.0, 3.0], &[3, 5])
        .with_transform(random_transform)
        .build();

    for (face_index, _face) in ref_shape.faces.iter().enumerate() {
        let uv_map = auto_uv_map_face(&ref_shape, face_index);
        let n = 100;
        let texture = draw_fuzz_on_uv(&uv_map, n);
        match texture {
            Texture::Lines { lines, .. } => {
                assert_eq!(lines.len(), n)
            }
            _ => panic!("Expected lines texture"),
        }
    }
}
