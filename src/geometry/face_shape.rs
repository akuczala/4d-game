use crate::colors::Color;
use crate::vector;
use crate::vector::{VectorTrait,MatrixTrait,Field,VecIndex};
use super::{Edge,Face,ShapeTrait,Plane,Shape};

use specs::{Component, DenseVecStorage};

#[derive(Clone,Component)]
#[storage(DenseVecStorage)]
pub struct FaceShape<V : VectorTrait> {
    pub verts_ref : Vec<V>,
    pub verts : Vec<V>,
    pub edges : Vec<Edge>,
    pub face : Face<V>,

    ref_frame : V::M,
    frame : V::M,
    pos : V,

    pub scale : Field,
    pub radius : Field,

}

impl<V : VectorTrait> FaceShape<V> {
    pub fn from_convex(shape : &Shape<V::SubV>, normal : &V) {
        
    }
    pub fn new(verts : Vec<V>, edges: Vec<Edge>, mut face: Face<V>) -> FaceShape<V> {

        face.calc_vertis(&edges);
        let face_verts = face.vertis.iter().map(|verti| verts[*verti]).collect();
        face.center_ref = vector::barycenter(&face_verts);
        //try to do this with iterators
        //face.center_ref = vector::barycenter_iter(&mut face.vertis.iter().map(|verti| verts[*verti]));
        face.center = face.center_ref.clone();

        let radius = FaceShape::calc_radius(&verts);
        let mut shape = FaceShape{
        verts_ref : verts.clone(),
        verts : verts,
        edges : edges,
        face : face,
        
        ref_frame : V::M::id(),
        frame : V::M::id(),
        pos : V::zero(),
        scale : 1.0,
        radius : radius,
        };
        shape.update();
        shape
    }
}
impl<V : VectorTrait> ShapeTrait<V> for FaceShape<V> {
    fn transform(&mut self) {
        for (v,vr) in self.verts.iter_mut().zip(self.verts_ref.iter()) {
            *v = self.frame * (*vr * self.scale) + self.pos;
        }
        self.face.normal = self.frame * self.face.normal_ref;
        self.face.threshold = self.face.normal.dot(self.pos);
    }
    fn update(&mut self) {
        self.transform();
    }
    fn rotate(&mut self, axis1: VecIndex, axis2: VecIndex, angle : Field) {
        let rot_mat = vector::rotation_matrix(self.frame[axis1],self.frame[axis2],Some(angle));
        self.frame = self.frame.dot(rot_mat);
        self.update();
    }
    fn set_pos(mut self, pos : &V) -> Self {
        self.pos = *pos;

        self.update();
        self
    }
    fn get_pos(& self) -> &V {
        &self.pos
    }
    fn stretch(&self, scales : &V) -> Self {
        let mut new_shape = self.clone();
        let new_verts : Vec<V> = self.verts_ref.iter()
            .map(|v| v.zip_map(*scales,|vi,si| vi*si)).collect();
        //need to explicitly update this as it stands
        //need to have a clear differentiation between
        //changes to mesh (verts_ref and center_ref) and
        //changes to position/orientation/scaling of mesh

        new_shape = new_shape.set_pos(&vector::barycenter(&self.verts));
        new_shape.radius = FaceShape::calc_radius(&new_verts);
        new_shape.verts_ref = new_verts;
        new_shape.update();
        new_shape
    }
    fn update_visibility(&mut self, camera_pos : V, transparent : bool) {
        if !transparent {
            self.face.update_visibility(camera_pos);
        }

    }
    fn set_color(mut self, color : Color) -> Self {
        self.face.set_color(color);
        self
    }
    fn calc_radius(verts : &Vec<V>) -> Field {
        verts.iter().map(|v| v.norm_sq()).fold(0./0., Field::max).sqrt()
    }
    fn calc_boundaries(&self, origin : V) -> Vec<Plane<V>> {
        todo!()
    }

}
