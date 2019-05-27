use crate::vector::{VectorTrait,MatrixTrait,Field,VecIndex,rotation_matrix};
use crate::geometry::{VertIndex,Shape};
use crate::graphics;

const Z0 : Field = 0.0;
const SMALL_Z : Field = 0.001;

pub struct Camera<V>
where V : VectorTrait
{
	pub pos : V,
	pub frame : V::M

}
impl<V> Camera<V>
where V : VectorTrait
{
	fn look_at(&mut self, point : V) {
		self.frame = rotation_matrix(V::one_hot(-1),point - self.pos,None);
	}
	fn update_frame(&mut self, axis1 : VecIndex, axis2 : VecIndex, angle : Field) {
		self.frame = rotation_matrix(self.frame[axis1], self.frame[axis2], Some(angle));
	}
}
fn project<V>(v : V) -> V::SubV
where V : VectorTrait
{
	let z;
	let focal : Field = 6.0;
	if V::is_close(v,V::ones()*Z0) {
		z = Z0 + SMALL_Z;
	} else {
		z = v[-1];
	}
	v.project()*focal/z
}
fn view_transform<V>(camera : &Camera<V>, point : V) -> V
where V : VectorTrait
{
	camera.frame.transpose() * point
}
fn draw<V>(display : &glium::Display, camera : &Camera<V>, shape : Shape<V>)
where V : VectorTrait
{
	//draw_wireframe(display,camera,shape)
}

pub fn draw_wireframe<V>(display : &glium::Display,
	camera : &Camera<V>,
	shape : Shape<V>) -> (Vec<V::SubV>,Vec<VertIndex>)
where V: VectorTrait
{
	//concatenate vertex indices from each edge to get list
	//of indices for drawing lines
	let mut vertis : Vec<VertIndex> = Vec::new(); 
    for edge in shape.edges.iter() {
        vertis.push(edge.0);
        vertis.push(edge.1);
    }
    let verts = shape.verts;
    let view_verts = verts.iter().map(|v| view_transform(camera,*v));
    let proj_verts : Vec<V::SubV> = view_verts.map(|v| project(v)).collect();
    for v in proj_verts.iter() {
    	println!("{}", v);
    }
    (proj_verts, vertis)
    //graphics::draw_lines(display,proj_verts,vertis);

}