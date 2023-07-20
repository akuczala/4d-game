use std::iter;
use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

use super::{face, Face, FaceIndex, Shape, VertIndex};
use crate::components::ShapeType;
use crate::constants::ZERO;
use crate::geometry::shape::single_face;
use crate::geometry::{line_plane_intersect, Line, Plane};
use crate::vector::{barycenter_iter, Field, VecIndex, VectorTrait};

#[derive(Clone, Serialize, Deserialize)]
pub struct BoundarySubFace<V> {
    pub vertis: Vec<VertIndex>, // list of vertis in each subface
    plane: Plane<V>, // this is not used for clipping, but is used for collisions + line intersection (e.g. targeting)
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
            subface.update(shape_vers, shape_faces[0].normal())
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
pub fn make_3d_triangle() -> Shape<Vec3> {
    Shape::new_single_face(
        vec![
            Vec3::new(1., -1., 1.),
            Vec3::new(1., 1., 1.),
            Vec3::new(-1., 0., 1.),
        ],
        vec![Edge(0, 1), Edge(1, 2), Edge(2, 0)],
        Face::new(vec![0, 1, 2], Vec3::new(0., 0., -1.)),
        &[vec![0, 1], vec![1, 2], vec![2, 0]],
    )
}
pub fn make_3d_square() -> Shape<Vec3> {
    Shape::new_single_face(
        vec![
            Vec3::new(-1., -1., 1.),
            Vec3::new(-1., 1., 1.),
            Vec3::new(1., -1., 1.),
            Vec3::new(1., 1., 1.),
        ],
        vec![Edge(0, 1), Edge(0, 2), Edge(1, 3), Edge(2, 3)],
        Face::new(vec![0, 1, 2, 3], Vec3::new(0., 0., -1.)),
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
