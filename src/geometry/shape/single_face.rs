use crate::vector::{VectorTrait,barycenter_iter,barycenter,is_close};
use crate::geometry::{Plane};
use super::{VertIndex,Face,Shape};

struct SubFace<V: VectorTrait>{
    vertis: Vec<VertIndex>, // list of vertis in each subface
    plane: Plane<V>
}
impl<V: VectorTrait> SubFace<V> {
    pub fn new(vertis: &Vec<VertIndex>, shape_verts: &Vec<V>, face_normal: V) -> Self {
        Self{
            vertis: vertis.clone(),
            plane: Self::calc_plane(vertis, shape_verts, face_normal),
        }
    }
    fn calc_plane(vertis: &Vec<VertIndex>, shape_verts: &Vec<V>, face_normal: V) -> Plane<V> {
        // take D-1 vertices of the subface, then subtract one of these from the others to get
        // D-2 vectors parallel to the subface
        let mut verts = vertis.iter()
            .take((V::DIM.abs() as usize) -1)
            .map(|&vi| shape_verts[vi]);
        let v0: V = verts.next().unwrap();
        let parallel_vecs = verts.map(|v| v - v0);
        let mut normal = V::cross_product(parallel_vecs.chain(std::iter::once(face_normal))).normalize();
        let shape_center = barycenter_iter(shape_verts.iter());
        if normal.dot(v0 - shape_center) < 0.0 { //normal should be pointing outward from center
            normal = -normal;
        }
        let subface_center = barycenter_iter(vertis.iter().map(|&vi| &shape_verts[vi]));
        Plane::from_normal_and_point(normal, subface_center)
    }
}
struct SubFaces<V: VectorTrait>(Vec<SubFace<V>>);

pub struct SingleFace<V: VectorTrait>{subfaces: SubFaces<V>}
impl<V: VectorTrait> SingleFace<V>{
    pub fn new(shape: &Shape<V>, subface_vertis: &Vec<Vec<VertIndex>>) -> Self {
        Self{
            subfaces: SubFaces(
                subface_vertis.iter().map(
                    |vertis| SubFace::new(vertis, &shape.verts, shape.faces[0].normal)
                ).collect()
            )
        }
    }
    fn calc_boundary(&self, subface: &SubFace<V>, origin: V, verts: &Vec<V>, face_center: V) -> Plane<V> {
        let mut boundary_normal = V::cross_product(
            subface.vertis.iter().take((V::DIM.abs() as usize) -1)
                .map(|&vi| verts[vi] - origin)
        ).normalize();
        //not sure about the sign here
        if boundary_normal.dot(face_center - origin) > 0. {
            boundary_normal = -boundary_normal;
        }
        Plane{ normal: boundary_normal, threshold: boundary_normal.dot(origin) }
    }
    pub fn calc_boundaries(&self, origin : V, verts: &Vec<V>, face_center: V) -> Vec<Plane<V>> {
        self.subfaces.0.iter().map(|subface| self.calc_boundary(subface, origin, verts, face_center)).collect()
    }
}

use crate::vector::{Vec2, Vec3};
use super::Edge;
fn make_3d_triangle() -> (Shape<Vec3>, SingleFace<Vec3>) {
    let shape = Shape::new(
        vec![Vec3::new(1.,-1.,1.),Vec3::new(1.,1.,1.),Vec3::new(-1.,0.,1.)],
        vec![Edge(0,1),Edge(1,2),Edge(2,0)],
        vec![Face::new(vec![0,1,2],Vec3::new(0.,0., -1.))]
    );
    let subfaces_vertis = vec![vec![0,1],vec![1,2],vec![2,0]];
    let single_face = SingleFace::new(&shape, &subfaces_vertis);
    (shape, single_face)
}
fn make_3d_square() -> (Shape<Vec3>, SingleFace<Vec3>) {
    let shape = Shape::new(
        vec![Vec3::new(-1.,-1.,1.),Vec3::new(-1.,1.,1.),Vec3::new(1.,-1.,1.),Vec3::new(1.,1.,1.)],
        vec![Edge(0,1),Edge(0,2),Edge(1,3),Edge(2,3)],
        vec![Face::new(vec![0,1,2,3],Vec3::new(0.,0., -1.))]
    );
    let subfaces_vertis = vec![vec![0,1],vec![0,2],vec![1,3],vec![2,3]];
    let single_face = SingleFace::new(&shape, &subfaces_vertis);
    (shape, single_face)
}
#[test]
fn test_boundaries() {
    let shape = Shape::new(
        vec![Vec2::new(1.,-1.),Vec2::new(1.,1.)],
        vec![Edge(0,1)],
        vec![Face::new(vec![0],Vec2::new(-1.,0.))]
    );
    let subfaces_vertis = vec![vec![0],vec![1]];
    let single_face = SingleFace::new(&shape, &subfaces_vertis);
    let boundaries = single_face.calc_boundaries(Vec2::zero(), &shape.verts, shape.faces[0].center);
    for boundary in boundaries.iter() {
        println!("{}",boundary)
    }
    println!("3d, Triangle");
    let (shape, single_face) = make_3d_triangle();
    let boundaries = single_face.calc_boundaries(Vec3::zero(), &shape.verts, shape.faces[0].center);
    for boundary in boundaries.iter() {
        assert!(is_close(boundary.threshold,0.0));
        //needs more asserts
        println!("{}",boundary)
    }
    println!("3d, Square");
    let (shape, single_face) = make_3d_square();
    let boundaries = single_face.calc_boundaries(Vec3::zero(), &shape.verts, shape.faces[0].center);
    for boundary in boundaries.iter() {
        assert!(is_close(boundary.threshold,0.0));
        //needs more asserts
        println!("{}",boundary)
    }
}

#[test]
fn test_subface_planes() {
    let (shape, single_face) = make_3d_square();
    type v = Vec3;
    let expected_normals = vec![-v::one_hot(0), -v::one_hot(1), v::one_hot(1), v::one_hot(0)];
    for (subface, &expected_normal) in single_face.subfaces.0.iter().zip(expected_normals.iter()) {
        assert!(is_close(subface.plane.threshold,1.0),"th={}",subface.plane.threshold);
        assert!(Vec3::is_close(subface.plane.normal, expected_normal),"normal={}",subface.plane.normal);
    }

    let (shape, single_face) = make_3d_triangle();
    let expected_planes = vec![
        Plane::from_normal_and_point(v::one_hot(0), v::one_hot(0)),
        Plane::from_normal_and_point(v::new(-1.0, 2.0,0.0).normalize(), v::new( 0.0, 0.5,0.0)),
        Plane::from_normal_and_point(v::new(-1.0,-2.0,0.0).normalize(), v::new(0.0, -0.5,0.0)),

    ];
    for (subface, expected_plane) in single_face.subfaces.0.iter().zip(expected_planes.iter()) {
        assert!(is_close(subface.plane.threshold,expected_plane.threshold),"th={}",expected_plane.threshold);
        assert!(Vec3::is_close(subface.plane.normal, expected_plane.normal),"normal={}",subface.plane.normal);

    }
}