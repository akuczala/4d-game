use std::iter;
use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

use super::face::FaceBuilder;
use super::subface::SubFace;
use super::{face, Face, FaceIndex, Shape, VertIndex};
use crate::components::{ShapeType, Transform};
use crate::constants::ZERO;
use crate::geometry::shape::single_face;
use crate::geometry::{line_plane_intersect, Line, Plane};
use crate::tests::utils::{color_number, print_grid};
use crate::vector::{barycenter_iter, Field, VecIndex, VectorTrait};

#[derive(Clone, Serialize, Deserialize)]
pub struct BoundarySubFace<V> {
    pub vertis: Vec<VertIndex>, // list of vertis in each subface
    pub plane: Plane<V>, // this is not used for clipping, but is used for collisions + line intersection (e.g. targeting)
    pub facei: FaceIndex,
}
impl<V: VectorTrait> BoundarySubFace<V> {
    pub fn new(vertis: &[VertIndex], shape_verts: &[V], face_normal: V, facei: FaceIndex) -> Self {
        Self {
            vertis: vertis.to_owned(),
            plane: Self::calc_plane(vertis, shape_verts, face_normal),
            facei,
        }
    }
    pub fn update(&mut self, shape_verts: &[V], face_normal: V) {
        self.plane = Self::calc_plane(&self.vertis, shape_verts, face_normal)
    }
    fn calc_plane(vertis: &[VertIndex], shape_verts: &[V], face_normal: V) -> Plane<V> {
        //note: would like to use some of the logic in Plane::calc_plane but here there are differences
        // take D-1 vertices of the subface, then subtract one of these from the others to get
        // D-2 vectors parallel to the subface
        let mut verts = vertis
            .iter()
            .take((V::DIM.unsigned_abs() - 1) as usize)
            .map(|&vi| shape_verts[vi]);
        let v0: V = verts.next().unwrap();
        let parallel_vecs = verts.map(|v| v - v0);
        let mut normal =
            V::cross_product(parallel_vecs.chain(std::iter::once(face_normal))).normalize();
        let shape_center = barycenter_iter(shape_verts.iter());
        if normal.dot(v0 - shape_center) < 0.0 {
            //normal should be pointing outward from center
            normal = -normal;
        }
        let subface_center = barycenter_iter(vertis.iter().map(|&vi| &shape_verts[vi]));
        Plane::from_normal_and_point(normal, subface_center)
    }
}
#[derive(Clone, Serialize, Deserialize)]
pub struct SubFaces<V>(pub Vec<BoundarySubFace<V>>);

#[derive(Clone, Serialize, Deserialize)]
pub struct SingleFace<V> {
    pub subfaces: SubFaces<V>,
}
impl<V: VectorTrait> SingleFace<V> {
    pub fn new(
        shape_verts: &[V],
        face_normal: V,
        subface_vertis: &[Vec<VertIndex>],
        face_index: FaceIndex,
    ) -> Self {
        Self {
            subfaces: SubFaces(
                subface_vertis
                    .iter()
                    .map(|vertis| {
                        BoundarySubFace::new(vertis, shape_verts, face_normal, face_index)
                    })
                    .collect(),
            ),
        }
    }
    pub fn update(&mut self, shape_vers: &[V], shape_faces: &[Face<V>]) {
        for subface in self.subfaces.0.iter_mut() {
            subface.update(shape_vers, shape_faces[subface.facei].normal())
        }
    }
    //returns points of intersection with shape
    pub fn line_intersect(
        subfaces: &SubFaces<V>,
        shape: &Shape<V>,
        line: &Line<V>,
        visible_only: bool,
        face_visibility: &[bool],
    ) -> Vec<V> {
        //impl std::iter::Iterator<Item=Option<V>> {
        let mut out_points = Vec::<V>::new();
        let face = &shape.faces[0];
        if !visible_only || face_visibility[0] {
            if let Some(p) = line_plane_intersect(line, face.plane()) {
                if SingleFace::subface_normal_distance(subfaces, p).1 < 0.0 {
                    out_points.push(p)
                }
            }
        }
        out_points
    }
    //returns distance to nearest subface plane
    pub fn subface_normal_distance(subfaces: &SubFaces<V>, pos: V) -> (V, Field) {
        let (closest_subshape_plane, distance) =
            Plane::point_normal_distance(pos, subfaces.0.iter().map(|sf| &sf.plane)).unwrap();
        (closest_subshape_plane.normal, distance)
    }
}

use super::Edge;
use crate::vector::{Vec2, Vec3};
pub fn make_line_shape() -> Shape<Vec2> {
    // TODO: add shape builder functionality for this
    let verts = vec![Vec2::new(1., -1.), Vec2::new(1., 1.)];
    let edges = vec![Edge(0, 1)];
    let face = FaceBuilder::new(&verts, &edges).build(vec![0], Vec2::new(-1., 0.), false);
    Shape::new_single_face(verts, edges, face, &[vec![0], vec![1]])
}

pub fn make_3d_triangle() -> Shape<Vec3> {
    let verts = vec![
        Vec3::new(1., -1., 1.),
        Vec3::new(1., 1., 1.),
        Vec3::new(-1., 0., 1.),
    ];
    let edges = vec![Edge(0, 1), Edge(1, 2), Edge(2, 0)];
    let face = FaceBuilder::new(&verts, &edges).build(vec![0, 1, 2], Vec3::new(0., 0., -1.), false);
    Shape::new_single_face(verts, edges, face, &[vec![0, 1], vec![1, 2], vec![2, 0]])
}
pub fn make_3d_square() -> Shape<Vec3> {
    let verts = vec![
        Vec3::new(-1., -1., 1.),
        Vec3::new(-1., 1., 1.),
        Vec3::new(1., -1., 1.),
        Vec3::new(1., 1., 1.),
    ];
    let edges = vec![Edge(0, 1), Edge(0, 2), Edge(1, 3), Edge(2, 3)];
    let face =
        FaceBuilder::new(&verts, &edges).build(vec![0, 1, 2, 3], Vec3::new(0., 0., -1.), false);
    Shape::new_single_face(
        verts,
        edges,
        face,
        &[vec![0, 1], vec![0, 2], vec![1, 3], vec![2, 3]],
    )
}

#[test]
fn test_subface_planes() {
    use crate::vector::is_close;
    let shape = make_3d_square();
    let single_face = match shape.shape_type {
        ShapeType::SingleFace(f) => f,
        _ => panic!("Expected single face variant"),
    };
    type V = Vec3;
    let expected_normals = vec![-V::one_hot(0), -V::one_hot(1), V::one_hot(1), V::one_hot(0)];
    for (subface, &expected_normal) in single_face.subfaces.0.iter().zip(expected_normals.iter()) {
        assert!(
            is_close(subface.plane.threshold, 1.0),
            "th={}",
            subface.plane.threshold
        );
        assert!(
            Vec3::is_close(subface.plane.normal, expected_normal),
            "normal={}",
            subface.plane.normal
        );
    }

    let shape = make_3d_triangle();
    let single_face = match shape.shape_type {
        ShapeType::SingleFace(f) => f,
        _ => panic!("Expected single face variant"),
    };
    let expected_planes = vec![
        Plane::from_normal_and_point(V::one_hot(0), V::one_hot(0)),
        Plane::from_normal_and_point(V::new(-1.0, 2.0, 0.0).normalize(), V::new(0.0, 0.5, 0.0)),
        Plane::from_normal_and_point(V::new(-1.0, -2.0, 0.0).normalize(), V::new(0.0, -0.5, 0.0)),
    ];
    for (subface, expected_plane) in single_face.subfaces.0.iter().zip(expected_planes.iter()) {
        assert!(
            is_close(subface.plane.threshold, expected_plane.threshold),
            "th={}",
            expected_plane.threshold
        );
        assert!(
            Vec3::is_close(subface.plane.normal, expected_plane.normal),
            "normal={}",
            subface.plane.normal
        );
    }
}
#[test]
fn test_subface_dist() {
    use crate::vector::is_close;
    let shape = make_3d_square();
    let single_face = match shape.shape_type {
        ShapeType::SingleFace(f) => f,
        _ => panic!("Expected single face variant"),
    };
    let (n, d) =
        SingleFace::subface_normal_distance(&single_face.subfaces, Vec3::new(0.5, 0.0, 0.0));
    assert!(Vec3::is_close(n, Vec3::one_hot(0)), "n={}", n);
    assert!(is_close(d, -0.5), "d={}", d);
    let (n, d) =
        SingleFace::subface_normal_distance(&single_face.subfaces, Vec3::new(0.5, 2.0, 0.0));
    assert!(Vec3::is_close(n, Vec3::one_hot(1)), "n={}", n);
    assert!(is_close(d, 1.0), "d={}", d);
}

#[test]
fn test_point_within2() {
    use crate::vector::linspace;
    use colored::*;

    fn point_normal_distance_i<'a, V: VectorTrait, I: Iterator<Item = &'a Plane<V>>>(
        point: V,
        planes: I,
    ) -> Option<(&'a Plane<V>, usize, Field)> {
        planes.enumerate().fold(
            None,
            |acc: Option<(&Plane<V>, usize, Field)>, (i, plane)| {
                let this_dist: Field = plane.point_signed_distance(point);
                Some(acc.map_or_else(
                    || (plane, i, this_dist),
                    |(best_plane, best_i, cur_dist)| match this_dist > cur_dist {
                        true => (plane, i, this_dist),
                        false => (best_plane, best_i, cur_dist),
                    },
                ))
            },
        )
    }
    pub fn subface_normal_distance_i<V: VectorTrait>(
        subfaces: &[SubFace<V>],
        pos: V,
    ) -> (V, usize, Field) {
        let (closest_subshape_plane, i, distance) = point_normal_distance_i(
            pos,
            subfaces.iter().map(|sf| match sf {
                SubFace::Convex(_) => panic!("Expected boundary"),
                SubFace::Boundary(bsf) => &bsf.plane,
            }),
        )
        .unwrap();
        (closest_subshape_plane.normal, i, distance)
    }

    let mut shape = make_line_shape();
    shape.modify(&Transform::identity().with_rotation(0, 1, 0.2));
    print_grid(2.0, 40, |x, y| {
        let point = Vec2::new(x, y);
        // let newstr = match shape.point_within(Vec3::new(x,y,0.),0.) {
        //   true => "+", false => "_"
        // };
        let (_, i, dist) = subface_normal_distance_i(&shape.shape_type.get_subfaces(), point);
        //println!("{}",dist);
        //let newstr = match dist {a if a > 1. => "#", a if a > 0. => "+", a if a <= 0. => "_", _ => "^"};
        let mut newstr = color_number(i);
        if dist > 1. {
            newstr = "+".to_string().white();
        }
        newstr
    })
    //assert!(false); //forces cargo test to print this
    //assert!(!shape.point_within(point,0.))
}
