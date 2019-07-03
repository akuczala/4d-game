use crate::vector::{VectorTrait,MatrixTrait,Field,VecIndex,rotation_matrix};
use crate::geometry::{VertIndex,Shape,Line,Plane,Face,Edge};
//use crate::graphics;
use crate::clipping::clip_line_plane;
use crate::colors::Color;
use crate::clipping::ClipState;

use itertools:: 	Itertools;
const Z0 : Field = 0.0;

const SMALL_Z : Field = 0.001;
const Z_NEAR : Field = 0.1; 
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
	const SPEED : Field = 2.0;
	const ANG_SPEED : Field = 2.0*3.14159/3.0;
	pub fn new(pos : V) -> Camera<V> {
		Camera{
			pos,
			frame : V::M::id(),
			heading : V::one_hot(-1),
			plane : Plane{normal : V::one_hot(-1), threshold : V::one_hot(-1).dot(pos)},

		}
	}
	pub fn look_at(&mut self, point : &V) {
		//self.frame = rotation_matrix(V::one_hot(-1),*point - self.pos,None).transpose();
		self.frame = rotation_matrix(*point - self.pos, V::one_hot(-1), None);
		self.update();
	}
	pub fn slide(&mut self, direction : V, time : Field) {
		self.pos = self.pos + direction.normalize()*Self::SPEED*time;
		self.update();
	}
	pub fn rotate(&mut self, axis1 : VecIndex, axis2 : VecIndex, speed_mult : Field) {
		self.frame = self.frame.dot(
			rotation_matrix(
			self.frame[axis1], self.frame[axis2],
			Some(speed_mult*Self::ANG_SPEED)));
		self.update();
	}
	pub fn update_plane(&mut self) {
		self.plane = Plane{
			normal : self.frame[-1],
			threshold : self.frame[-1].dot(self.pos)
		}
	}
	pub fn update_heading(&mut self) {
		self.heading = self.frame[-1];
	}
	pub fn update(&mut self) {
		self.update_heading();
		self.update_plane();
	}
}

pub struct DrawVertex<V>
where V: VectorTrait
{
	pub vertex : V,
	pub color : Color
}

pub struct DrawLine<V>
where V : VectorTrait
{
	pub line : Line<V>,
	pub color : Color,
}
impl<V : VectorTrait> DrawLine<V> {
	pub fn map_line<F,U>(self, f : F) -> DrawLine<U>
	where U : VectorTrait,
	F : Fn(Line<V>) -> Line<U>
	{
		DrawLine{
			line : f(self.line),
			color : self.color
		}
	}
	pub fn get_draw_verts(&self) -> (DrawVertex<V>,DrawVertex<V>) {
		(DrawVertex{
			vertex : self.line.0,
			color : self.color
		},
		DrawVertex{
			vertex : self.line.1,
			color : self.color
		})
	}
}

pub enum Texture<V : VectorTrait> {
	DefaultLines,
	DrawLines(Vec<DrawLine<V>>),
	Lines(Vec<Line<V>>)
}
impl<V: VectorTrait> Texture<V> {
	pub fn make_tile_texture(scales : &Vec<Field>, n_divisions : &Vec<i32>) -> Vec<Line<V>> {
		if V::DIM != n_divisions.len() as VecIndex {
			panic!("make_tile_texture: Expected n_divisions.len()={} but got {}", V::DIM, n_divisions.len());
		}

		let centers= n_divisions.iter().map(|n| (0..*n))
		.multi_cartesian_product().map(|ivec|
			ivec.iter().enumerate()
			.map(|(axis,&i)| V::one_hot(axis as VecIndex)*((i as Field) + 0.5)/((n_divisions[axis]) as Field))
			.fold(V::zero(),|v,u| v + u)
			);

		//all this does is convert n_divisions to a vector and divide by 2
		//but since i haven't bothered putting a Vec<Field> -> V function in the vector library 
		//i have to do this ridiculous fold
		//see also the computation of the centers
		let corner = n_divisions.iter().enumerate()
			.map(|(ax,&n)| V::one_hot(ax as VecIndex)/(n as Field))
			.fold(V::zero(),|v,u| v + u)/2.0;

		//an element-wise multiplication by corner would simplify things a bit here
		//zero-centered lines scaled by subdivision

		//this is wrong
		// let tile_lines : Vec<Line<V>> = n_divisions.iter().enumerate()
		// .map(|(ax,&n)| {
		// 	let frame_vec = V::one_hot(ax as VecIndex)/(n as Field);
		// 	vec![Line(-corner,-corner + frame_vec),Line(corner - frame_vec,corner)]
		// })
		// .flat_map(|lines| lines).collect(); //flat_map instead of flatten because of itertools/iterator conflict

		//grow edges starting from a line
		let mut tile_lines : Vec<Line<V>> = Vec::new();
		for (i,&n) in n_divisions.iter().enumerate() {
			if i == 0 {
				tile_lines.push(Line(-corner,-corner + V::one_hot(0)/(n as Field)));
			} else {
				let new_dir = V::one_hot(i as VecIndex)/(n as Field);
				let mut new_lines : Vec<Line<V>> = tile_lines.iter()
					.map(|line| vec![
						line.map(|v| v + new_dir),
						Line(line.0,line.0 + new_dir),
						Line(line.1,line.1 + new_dir)
						])
					.flat_map(|lines| lines).collect();

				tile_lines.append(&mut new_lines);
			}
		}
		

		centers.cartesian_product(scales.iter())
			.map(|(center,&scale)| tile_lines.iter()
				.map(|line| line.map(|v| v*scale + center))
				.collect())
			.flat_map(|lines : Vec<Line<V>>| lines)
			.collect()


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
	camera.frame * (point - camera.pos)
}
//this takes a line and returns Option<Line>
pub fn transform_line<V>(line : Line<V>, camera : &Camera<V>) -> Option<Line<V::SubV>>
where V : VectorTrait
{
	let clipped_line = clip_line_plane(line,&camera.plane,Z_NEAR);
	let view_line = clipped_line
		.map(|line| line
		.map(|v| view_transform(&camera,v)));
	let proj_line = view_line
		.map(|line| line
		.map(project));
	proj_line
}
//apply transform line to the lines in draw_lines
//need to do nontrivial destructuring and reconstructing
//in order to properly handle Option
//would probably benefit from something monad-like
pub fn transform_draw_lines<V>(
	draw_lines : Vec<Option<DrawLine<V>>>,
	camera : &Camera<V>) -> Vec<Option<DrawLine<V::SubV>>>
where V : VectorTrait
{
	draw_lines.into_iter()
		.map(|opt_draw_line| match opt_draw_line {
			Some(draw_line) => {
				let transformed_line = transform_line(draw_line.line,&camera);
				match transformed_line {
					Some(line) => Some(DrawLine{line, color : draw_line.color}),
					None => None
				}
			}
			None => None
		})
		.collect()
}
pub fn transform_lines<V>(lines : Vec<Option<Line<V>>>,
	camera : &Camera<V>) -> Vec<Option<Line<V::SubV>>>
where V : VectorTrait
{
	let clipped_lines = lines
	.into_iter()
	.map(|maybe_line| match maybe_line {
		Some(line) => clip_line_plane(line,&camera.plane,Z_NEAR),
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
pub fn calc_face_lines<V>(
	face : &Face<V>,
	shape : &Shape<V>,
	face_scales : &Vec<Field>
	) -> Vec<Option<DrawLine<V>>>
where V : VectorTrait {
	let mut shape_lines : Vec<Option<DrawLine<V>>> = Vec::with_capacity(face.edgeis.len()*face_scales.len());
	for &face_scale in face_scales {
		let scale_point = |v| V::linterp(face.center,v,face_scale);
		for edgei in &face.edgeis {
			let edge = &shape.edges[*edgei];
			//println!("{}",edge);
			if face.visible || shape.transparent {
				shape_lines.push(
					Some(DrawLine{
						line : Line(
						shape.verts[edge.0],
						shape.verts[edge.1])
						.map(scale_point),
						color : face.color
						})
				);
			} else {
				shape_lines.push(None);
			}
		}
	}
	shape_lines
}
pub fn update_shape_visibility<V : VectorTrait>(
	camera : &Camera<V>,
	shapes : &mut Vec<Shape<V>>,
	clip_state : &ClipState<V>
	) {
	//update shape visibility and boundaries
	for shape in shapes.iter_mut() {
		shape.update_visibility(camera.pos);
		//calculate boundaries for clipping
		if clip_state.clipping_enabled {
			shape.boundaries = crate::clipping::calc_boundaries(
				&shape.faces, &shape.subfaces, camera.pos);
		}
	}

}
pub fn calc_shapes_lines<V>(
	shapes : &mut Vec<Shape<V>>,
	face_scale : &Vec<Field>,
	clip_state : &ClipState<V>,
	)  -> Vec<Option<DrawLine<V>>>

where V : VectorTrait
{
	
	//probably worth computing / storing number of lines
	//and using Vec::with_capacity
	
	let mut lines : Vec<Option<DrawLine<V>>> = Vec::new();
	
	//compute lines for each shape
	for (shape,shape_in_front) in shapes.iter().zip(clip_state.in_front.iter()) {
		let mut shape_lines : Vec<Option<DrawLine<V>>> = Vec::new();
		//get lines from each face
		for face in &shape.faces {
			shape_lines.append(&mut calc_face_lines(face,&shape,&face_scale));
		}
		//clip these lines and append to list
		if clip_state.clipping_enabled {
			let mut clipped_lines = crate::clipping::clip_draw_lines(
				shape_lines, shapes, Some(shape_in_front));
			lines.append(&mut clipped_lines);
		} else {
			lines.append(&mut shape_lines);
		}
		
	}
	lines
	
}
pub fn calc_lines_color<V : VectorTrait>(
	shapes : &Vec<Shape<V>>,
	lines : Vec<Line<V>>,
	color : Color
	) -> Vec<Option<DrawLine<V>>> {

	let draw_lines = lines
		.into_iter()
		.map(|line| Some(DrawLine{line : line,color}))
		.collect();

	let clipped_lines = crate::clipping::clip_draw_lines(
			draw_lines, shapes, None);

	clipped_lines
}
//ehh. need to clone in here since we're borrowing lines
pub fn calc_lines_color_from_ref<V : VectorTrait>(
	shapes : &Vec<Shape<V>>,
	lines : &Vec<Line<V>>,
	color : Color
	) -> Vec<Option<DrawLine<V>>> {

	let draw_lines = lines
		.iter()
		.map(|line| Some(DrawLine{line : (*line).clone(),color}))
		.collect();

	let clipped_lines = crate::clipping::clip_draw_lines(
			draw_lines, shapes, None);

	clipped_lines
}

pub fn draw_wireframe<V>(//display : &glium::Display,
	shape : &Shape<V>, color : Color) -> Vec<Option<DrawLine<V>>>
where V: VectorTrait
{
	//concatenate vertex indices from each edge to get list
	//of indices for drawing lines
	let mut vertis : Vec<VertIndex> = Vec::new(); 
    for edge in shape.edges.iter() {
        vertis.push(edge.0);
        vertis.push(edge.1);
    }

    let lines : Vec<Option<Line<V>>> = vertis.chunks(2)
    	.map(|pair| Some(Line(shape.verts[pair[0]],shape.verts[pair[1]]))).collect();
    
    let draw_lines = lines.into_iter().map(|opt_line| opt_line
    	.map(|line| DrawLine{line,color})).collect();

    draw_lines

}
