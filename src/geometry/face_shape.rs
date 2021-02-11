use crate::colors::Color;
use crate::vector;
use crate::vector::{VectorTrait,MatrixTrait,Field,VecIndex,barycenter};
use super::{Edge,Face,VertIndex,ShapeTrait,Plane};

use specs::{Component, DenseVecStorage};

#[derive(Clone)]
pub struct SubFace(Vec<VertIndex>); // list of vertis in each subface

#[derive(Clone)]
pub struct FaceShape<V : VectorTrait> {
    pub verts_ref : Vec<V>,
    pub verts : Vec<V>,
    pub edges : Vec<Edge>,
    pub face : Face<V>,

    subfaces: Vec<SubFace>,

    ref_frame : V::M,
    frame : V::M,
    pos : V,

    pub scale : Field,

}

impl<V: VectorTrait> Component for FaceShape<V> {
        type Storage = DenseVecStorage<Self>;
}

impl<V : VectorTrait> FaceShape<V> {
    pub fn new(verts : Vec<V>, edges: Vec<Edge>, subfaces: Vec<SubFace>, mut face: Face<V>) -> FaceShape<V> {

        face.calc_vertis(&edges);
        let face_verts = face.vertis.iter().map(|verti| verts[*verti]).collect();
        face.center_ref = vector::barycenter(&face_verts);
        //try to do this with iterators
        //face.center_ref = vector::barycenter_iter(&mut face.vertis.iter().map(|verti| verts[*verti]));
        face.center = face.center_ref.clone();

        let mut shape = FaceShape{
            verts_ref : verts.clone(),
            verts: verts,
            edges: edges,
            face: face,

            subfaces,

            ref_frame : V::M::id(),
            frame : V::M::id(),
            pos : V::zero(),
            scale : 1.0,
        };
        shape
    }
    pub fn calc_boundary(&self, subface: &SubFace, origin: V) -> Plane<V> {
        let mut boundary_normal = V::cross_product(
            subface.0.iter().take((V::DIM.abs() as usize) -1)
                .map(|&vi| self.verts[vi] - origin)
        ).normalize();
        //not sure about the sign here
        if boundary_normal.dot(self.face.center - origin) < 0. {
            boundary_normal = -boundary_normal;
        }
        Plane{ normal: boundary_normal, threshold: boundary_normal.dot(origin) }
    }
    pub fn calc_boundaries(&self, origin : V) -> Vec<Plane<V>> {
        self.subfaces.iter().map(|subface| self.calc_boundary(subface, origin)).collect()
    }
    //returns distance and normal of closest face
    pub fn point_normal_distance(&self, point : V) -> (V, Field) {
        (self.face.normal, self.face.normal.dot(point) - self.face.threshold)
    }
}

#[test]
fn test_boundaries() {
    use vector::{Vec2, Vec3};
    let face_shape = FaceShape::new(
        vec![Vec2::new(1.,-1.),Vec2::new(1.,1.)],
        vec![Edge(0,1)],
        vec![SubFace(vec![0]),SubFace(vec![1])],
        Face::new(vec![0],Vec2::new(-1.,0.))
    );
    let boundaries = face_shape.calc_boundaries(Vec2::zero());
    for boundary in boundaries.iter() {
        println!("{}",boundary)
    }
    println!("3d, Triangle");
    let face_shape = FaceShape::new(
        vec![Vec3::new(1.,-1.,1.),Vec3::new(1.,1.,1.),Vec3::new(-1.,0.,1.)],
        vec![Edge(0,1),Edge(1,2),Edge(2,0)],
        vec![SubFace(vec![0,1]),SubFace(vec![1,2]),SubFace(vec![2,0])],
        Face::new(vec![0,1,2],Vec3::new(0.,0., -1.))
    );
    let boundaries = face_shape.calc_boundaries(Vec3::zero());
    for boundary in boundaries.iter() {
        println!("{}",boundary)
    }
    println!("3d, Square");
    let face_shape = FaceShape::new(
        vec![Vec3::new(-1.,-1.,1.),Vec3::new(-1.,1.,1.),Vec3::new(1.,-1.,1.),Vec3::new(1.,1.,1.)],
        vec![Edge(0,1),Edge(0,2),Edge(1,3),Edge(2,3)],
        vec![SubFace(vec![0,1]),SubFace(vec![0,2]),SubFace(vec![1,3]),SubFace(vec![2,3])],
        Face::new(vec![0,1,2,3],Vec3::new(0.,0., -1.))
    );
    let boundaries = face_shape.calc_boundaries(Vec3::zero());
    for boundary in boundaries.iter() {
        println!("{}",boundary)
    }



}
