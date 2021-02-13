use crate::vector::{VectorTrait};
use crate::geometry::{Plane};
use super::{VertIndex,Face,Shape};

struct SubFace<V: VectorTrait>{
    vertis: Vec<VertIndex>, // list of vertis in each subface
    plane: Plane<V>
}
impl<V: VectorTrait> SubFace<V> {
    pub fn new(vertis: &Vec<VertIndex>, shape_verts: &Vec<V>, shape_center: V, face_normal: V) {

    }
    fn calc_plane(vertis: &Vec<VertIndex>, shape_center: V, shape_verts: &Vec<V>, face_normal: V) -> Plane<V> {
        // take D-1 vertices of the subface, then subtract one of these from the others to get
        // D-2 vectors parallel to the subface
        let mut verts = vertis.iter()
            .take((V::DIM.abs() as usize) -1)
            .map(|&vi| verts[vi] - shape_center);
        let v0: V = *verts.next().unwrap();
        let parallel_vecs = verts.map(|&v| v - v0)
        let mut normal = V::cross_product(parallel_vecs.chain(std::iter::once(face_normal)));
        if normal.dot(v0 - shape_center) < 0.0 { //normal should be pointing outward from center
            normal = -normal;
        }
        Plane::from_normal_and_point(normal, point: barycenter of subface verts)



        let mut normal = V::cross_product(
            vertis.iter().take((V::DIM.abs() as usize) -1)
                .map(|&vi| verts[vi] - shape_center)
        ).normalize();
    }
}
struct SubFaces<V: VectorTrait>(Vec<SubFace<V>>);

pub struct SingleFace<V: VectorTrait>{subfaces: SubFaces<V>}
impl SingleFace{
    pub fn new<V: VectorTrait>(shape: &Shape<V>, subface_vertis: &Vec<Vec<VertIndex>>) -> Self {
        Self{
            subfaces: SubFaces(
                subface_vertis.iter().map(|vertis| SubFace(vertis.clone())).collect()
            )
        }
    }
    fn calc_boundary<V: VectorTrait>(&self, subface: &SubFace, origin: V, verts: &Vec<V>, face_center: V) -> Plane<V> {
        let mut boundary_normal = V::cross_product(
            subface.0.iter().take((V::DIM.abs() as usize) -1)
                .map(|&vi| verts[vi] - origin)
        ).normalize();
        //not sure about the sign here
        if boundary_normal.dot(face_center - origin) > 0. {
            boundary_normal = -boundary_normal;
        }
        Plane{ normal: boundary_normal, threshold: boundary_normal.dot(origin) }
    }
    pub fn calc_boundaries<V: VectorTrait>(&self, origin : V, verts: &Vec<V>, face_center: V) -> Vec<Plane<V>> {
        self.subfaces.0.iter().map(|subface| self.calc_boundary(subface, origin, verts, face_center)).collect()
    }
}

#[test]
fn test_boundaries() {
    use crate::vector::{Vec2, Vec3};
    use super::Edge;
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
    let shape = Shape::new(
        vec![Vec3::new(1.,-1.,1.),Vec3::new(1.,1.,1.),Vec3::new(-1.,0.,1.)],
        vec![Edge(0,1),Edge(1,2),Edge(2,0)],
        vec![Face::new(vec![0,1,2],Vec3::new(0.,0., -1.))]
    );
    let subfaces_vertis = vec![vec![0,1],vec![1,2],vec![2,0]];
    let single_face = SingleFace::new(&shape, &subfaces_vertis);
    let boundaries = single_face.calc_boundaries(Vec3::zero(), &shape.verts, shape.faces[0].center);
    for boundary in boundaries.iter() {
        println!("{}",boundary)
    }
    println!("3d, Square");
    let shape = Shape::new(
        vec![Vec3::new(-1.,-1.,1.),Vec3::new(-1.,1.,1.),Vec3::new(1.,-1.,1.),Vec3::new(1.,1.,1.)],
        vec![Edge(0,1),Edge(0,2),Edge(1,3),Edge(2,3)],
        vec![Face::new(vec![0,1,2,3],Vec3::new(0.,0., -1.))]
    );
    let subfaces_vertis = vec![vec![0,1],vec![0,2],vec![1,3],vec![2,3]];
    let single_face = SingleFace::new(&shape, &subfaces_vertis);
    let boundaries = single_face.calc_boundaries(Vec3::zero(), &shape.verts, shape.faces[0].center);
    for boundary in boundaries.iter() {
        println!("{}",boundary)
    }



}