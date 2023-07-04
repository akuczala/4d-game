use super::{Face, Shape, VertIndex};
use crate::geometry::{line_plane_intersect, Line, Plane};
use crate::vector::{barycenter_iter, Field, VectorTrait};

#[derive(Clone)]
struct SubFace<V> {
    vertis: Vec<VertIndex>, // list of vertis in each subface
    plane: Plane<V>,
}
impl<V: VectorTrait> SubFace<V> {
    pub fn new(vertis: &Vec<VertIndex>, shape_verts: &Vec<V>, face_normal: V) -> Self {
        Self {
            vertis: vertis.clone(),
            plane: Self::calc_plane(vertis, shape_verts, face_normal),
        }
    }
    pub fn update(&mut self, shape_verts: &Vec<V>, face_normal: V) {
        self.plane = Self::calc_plane(&self.vertis, shape_verts, face_normal)
    }
    fn calc_plane(vertis: &Vec<VertIndex>, shape_verts: &Vec<V>, face_normal: V) -> Plane<V> {
        //note: would like to use some of the logic in Plane::calc_plane but here there are differences
        // take D-1 vertices of the subface, then subtract one of these from the others to get
        // D-2 vectors parallel to the subface
        let mut verts = vertis
            .iter()
            .take((V::DIM.abs() as usize) - 1)
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
#[derive(Clone)]
struct SubFaces<V>(Vec<SubFace<V>>);

#[derive(Clone)]
pub struct SingleFace<V> {
    subfaces: SubFaces<V>,
    pub two_sided: bool,
}
impl<V: VectorTrait> SingleFace<V> {
    pub fn new(shape: &Shape<V>, subface_vertis: &Vec<Vec<VertIndex>>, two_sided: bool) -> Self {
        Self {
            subfaces: SubFaces(
                subface_vertis
                    .iter()
                    .map(|vertis| SubFace::new(vertis, &shape.verts, shape.faces[0].normal()))
                    .collect(),
            ),
            two_sided,
        }
    }
    pub fn update(&mut self, shape: &Shape<V>) {
        for subface in self.subfaces.0.iter_mut() {
            subface.update(&shape.verts, shape.faces[0].normal())
        }
    }
    fn calc_boundary(
        &self,
        subface: &SubFace<V>,
        origin: V,
        verts: &Vec<V>,
        face_center: V,
    ) -> Plane<V> {
        let mut boundary_normal = V::cross_product(
            subface
                .vertis
                .iter()
                .take((V::DIM.abs() as usize) - 1)
                .map(|&vi| verts[vi] - origin),
        )
        .normalize();
        //not sure about the sign here
        if boundary_normal.dot(face_center - origin) > 0. {
            boundary_normal = -boundary_normal;
        }
        Plane {
            normal: boundary_normal,
            threshold: boundary_normal.dot(origin),
        }
    }
    pub fn calc_boundaries(
        &self,
        origin: V,
        verts: &Vec<V>,
        face_center: V,
        visible: bool,
    ) -> Vec<Plane<V>> {
        if visible {
            self.subfaces
                .0
                .iter()
                .map(|subface| self.calc_boundary(subface, origin, verts, face_center))
                .collect()
        } else {
            Vec::new()
        }
    }
    //returns points of intersection with shape
    pub fn line_intersect(
        &self,
        shape: &Shape<V>,
        line: &Line<V>,
        visible_only: bool,
        face_visibility: &Vec<bool>,
    ) -> Vec<V> {
        //impl std::iter::Iterator<Item=Option<V>> {
        let mut out_points = Vec::<V>::new();
        let face = &shape.faces[0];
        if !visible_only || face_visibility[0] {
            if let Some(p) = line_plane_intersect(line, face.plane()) {
                if self.subface_normal_distance(p).1 < 0.0 {
                    out_points.push(p)
                }
            }
        }
        out_points
    }
    //returns distance to nearest subface plane
    pub fn subface_normal_distance(&self, pos: V) -> (V, Field) {
        let (closest_subshape_plane, distance) =
            Plane::point_normal_distance(pos, self.subfaces.0.iter().map(|sf| &sf.plane)).unwrap();
        (closest_subshape_plane.normal, distance)
    }
}

use super::Edge;
use crate::vector::{Vec2, Vec3};
fn make_3d_triangle() -> (Shape<Vec3>, SingleFace<Vec3>) {
    let shape = Shape::new(
        vec![
            Vec3::new(1., -1., 1.),
            Vec3::new(1., 1., 1.),
            Vec3::new(-1., 0., 1.),
        ],
        vec![Edge(0, 1), Edge(1, 2), Edge(2, 0)],
        vec![Face::new(vec![0, 1, 2], Vec3::new(0., 0., -1.))],
    );
    let subfaces_vertis = vec![vec![0, 1], vec![1, 2], vec![2, 0]];
    let single_face = SingleFace::new(&shape, &subfaces_vertis, false);
    (shape, single_face)
}
fn make_3d_square() -> (Shape<Vec3>, SingleFace<Vec3>) {
    let shape = Shape::new(
        vec![
            Vec3::new(-1., -1., 1.),
            Vec3::new(-1., 1., 1.),
            Vec3::new(1., -1., 1.),
            Vec3::new(1., 1., 1.),
        ],
        vec![Edge(0, 1), Edge(0, 2), Edge(1, 3), Edge(2, 3)],
        vec![Face::new(vec![0, 1, 2, 3], Vec3::new(0., 0., -1.))],
    );
    let subfaces_vertis = vec![vec![0, 1], vec![0, 2], vec![1, 3], vec![2, 3]];
    let single_face = SingleFace::new(&shape, &subfaces_vertis, false);
    (shape, single_face)
}
#[test]
fn test_boundaries() {
    use crate::vector::is_close;
    let shape = Shape::new(
        vec![Vec2::new(1., -1.), Vec2::new(1., 1.)],
        vec![Edge(0, 1)],
        vec![Face::new(vec![0], Vec2::new(-1., 0.))],
    );
    let subfaces_vertis = vec![vec![0], vec![1]];
    let single_face = SingleFace::new(&shape, &subfaces_vertis, false);
    let boundaries =
        single_face.calc_boundaries(Vec2::zero(), &shape.verts, shape.faces[0].center(), true);
    for boundary in boundaries.iter() {
        println!("{}", boundary)
    }
    println!("3d, Triangle");
    let (shape, single_face) = make_3d_triangle();
    let boundaries =
        single_face.calc_boundaries(Vec3::zero(), &shape.verts, shape.faces[0].center(), true);
    for boundary in boundaries.iter() {
        assert!(is_close(boundary.threshold, 0.0));
        //needs more asserts
        println!("{}", boundary)
    }
    println!("3d, Square");
    let (shape, single_face) = make_3d_square();
    let boundaries =
        single_face.calc_boundaries(Vec3::zero(), &shape.verts, shape.faces[0].center(), true);
    for boundary in boundaries.iter() {
        assert!(is_close(boundary.threshold, 0.0));
        //needs more asserts
        println!("{}", boundary)
    }
}

#[test]
fn test_subface_planes() {
    use crate::vector::is_close;
    let (_shape, single_face) = make_3d_square();
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

    let (_shape, single_face) = make_3d_triangle();
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
    let (_shape, single_face) = make_3d_square();
    let (n, d) = single_face.subface_normal_distance(Vec3::new(0.5, 0.0, 0.0));
    assert!(Vec3::is_close(n, Vec3::one_hot(0)), "n={}", n);
    assert!(is_close(d, -0.5), "d={}", d);
    let (n, d) = single_face.subface_normal_distance(Vec3::new(0.5, 2.0, 0.0));
    assert!(Vec3::is_close(n, Vec3::one_hot(1)), "n={}", n);
    assert!(is_close(d, 1.0), "d={}", d);
}
