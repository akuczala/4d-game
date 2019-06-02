use crate::vector::{VectorTrait,MatrixTrait,Field,VecIndex,rotation_matrix};
use crate::geometry::{VertIndex,Shape,Line,Plane};
use crate::graphics;
use crate::clipping::clip_line_plane;

const Z0 : Field = 0.0;
const SMALL_Z : Field = 0.001;

pub struct Camera<V>
where V : VectorTrait
{
	pub pos : V,
	pub frame : V::M,
	pub heading : V,
	pub plane : Plane<V>,

}
impl<V> Camera<V>
where V : VectorTrait
{
	const SPEED : Field = 0.05;
	const ANG_SPEED : Field = 0.01;
	pub fn new(pos : V) -> Camera<V> {
		Camera{
			pos,
			frame : V::M::id(),
			heading : V::one_hot(-1),
			plane : Plane{normal : V::one_hot(-1), threshold : V::one_hot(-1).dot(pos)}
		}
	}
	pub fn look_at(&mut self, point : V) {
		self.frame = rotation_matrix(V::one_hot(-1),point - self.pos,None);
		self.update_heading();
		self.update_plane();
	}
	pub fn slide(&mut self, direction : V) {
		self.pos = self.pos + direction.normalize()*Self::SPEED;
		self.update_plane();
	}
	pub fn rotate(&mut self, axis1 : VecIndex, axis2 : VecIndex, speed_mult : Field) {
		self.frame = rotation_matrix(
			self.frame[axis1], self.frame[axis2],
			Some(speed_mult*Self::ANG_SPEED)).dot(self.frame);
		self.update_heading();
		self.update_plane();
	}
	pub fn update_plane(&mut self) {
		self.plane = Plane{
			normal : self.frame[-1],
			threshold : self.frame.transpose()[-1].dot(self.pos)
		}
	}
	pub fn update_heading(&mut self) {
		self.heading = self.frame.transpose()[-1];
	}
}
fn project<V>(v : V) -> V::SubV
where V : VectorTrait
{
	let z;
	let focal : Field = 1.0;
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
	camera.frame.transpose() * (point - camera.pos)
}
pub fn transform_lines<V>(lines : Vec<Option<Line<V>>>,
	camera : &Camera<V>) -> Vec<Option<Line<V::SubV>>>
where V : VectorTrait
{
	let clipped_lines = lines
	.into_iter()
	.map(|maybe_line| match maybe_line {
		Some(line) => clip_line_plane(line,&camera.plane,2.0),
		None => None
	});
    //let clipped_lines = lines.map(|line| Some(line)); //no clipping
    let view_lines = clipped_lines
    	.map(|maybe_line| maybe_line
    		.map(|line| line
    			.map(|v| view_transform(&camera,v))));
    let proj_lines = view_lines
    	.map(|maybe_line| maybe_line
    		.map(|line| line
    			.map(project))).collect();
    proj_lines
}
pub fn draw_shape<V>(
	camera : &Camera<V>,
	shape : &Shape<V>)  -> Vec<Option<Line<V::SubV>>>

where V : VectorTrait
{
	let mut shape_lines : Vec<Option<Line<V>>> = Vec::new();

	//get lines from each face
	for face in &shape.faces {
		let scale_point = |v| V::linterp(face.center,v,0.8);
		for edgei in &face.edgeis {
			let edge = &shape.edges[*edgei];
			//println!("{}",edge);
			shape_lines.push(
				match face.visible {
					true => Some(Line(
						shape.verts[edge.0],
						shape.verts[edge.1])
					.map(scale_point)),
					false => None
				}
			);
		}
	}
	transform_lines(shape_lines,camera)
}

pub fn draw_wireframe<V>(//display : &glium::Display,
	camera : &Camera<V>,
	shape : &Shape<V>) -> Vec<Option<Line<V::SubV>>>
where V: VectorTrait
{
	//concatenate vertex indices from each edge to get list
	//of indices for drawing lines
	let mut vertis : Vec<VertIndex> = Vec::new(); 
    for edge in shape.edges.iter() {
        vertis.push(edge.0);
        vertis.push(edge.1);
    }

    // for pair in vertis.chunks(2) {
    // 	lines.push(Line(shape.verts[pair[0]],shape.verts[pair[1]]));
    // }
    let lines : Vec<Option<Line<V>>> = vertis.chunks(2)
    	.map(|pair| Some(Line(shape.verts[pair[0]],shape.verts[pair[1]]))).collect();
    
    transform_lines(lines, camera)
    //let view_verts = verts.iter().map(|v| view_transform(camera,*v));

    //let proj_verts : Vec<V::SubV> = view_verts.map(|v| project(v)).collect();
    // for v in proj_verts.iter() {
    // 	println!("{}", v);
    // }
    //(proj_verts, vertis)
    //graphics::draw_lines(display,proj_verts,vertis);

}