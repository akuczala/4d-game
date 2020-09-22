#[allow(dead_code)]
mod texture;

use crate::player::Player;
use specs::prelude::*;
use std::marker::PhantomData;

extern crate map_in_place;

pub use texture::Texture;
pub use texture::TextureMapping;

use crate::camera::{Camera};
use crate::vector::{VectorTrait,Field};
use crate::geometry::{VertIndex,Shape,Line};
//use crate::graphics;
use crate::clipping::{clip_line_plane,clip_line_sphere,clip_line_cube,ShapeClipState};
use crate::colors::*;
use crate::clipping::ClipState;

const Z0 : Field = 0.0;

const SMALL_Z : Field = 0.001;
const Z_NEAR : Field = 0.1; 

const CLIP_SPHERE_RADIUS : Field = 0.5;
#[derive(Clone,Copy)]
pub struct DrawVertex<V>
where V: VectorTrait
{
	pub vertex : V,
	pub color : Color
}

#[derive(Clone)]
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
	pub fn get_draw_verts(&self) -> [DrawVertex<V> ; 2] {
		[DrawVertex{
			vertex : self.line.0,
			color : self.color
		},
		DrawVertex{
			vertex : self.line.1,
			color : self.color
		}]
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
//this takes a Option<Line> and returns Option<Line>
pub fn transform_line<V>(line : Option<Line<V>>, camera : &Camera<V>) -> Option<Line<V::SubV>>
where V : VectorTrait
{
	let clipped_line = match line {Some(l) => clip_line_plane(l,&camera.plane,Z_NEAR), None => None};

	let view_line = clipped_line
		.map(|l| l
		.map(|v| view_transform(&camera,v)));
	let proj_line = view_line
		.map(|l| l
		.map(project));
	let clip_proj_line = match proj_line {Some(l) => clip_line_cube(l,CLIP_SPHERE_RADIUS), None => None};
	clip_proj_line
}

pub struct DrawLineList<V : VectorTrait>(pub Vec<Option<DrawLine<V>>>);
impl<V : VectorTrait> DrawLineList<V> {
	pub fn len(&self) -> usize {
		self.0.len()
	}
	pub fn map<F,U>(&self, f : F) -> DrawLineList<U>
	where U : VectorTrait,
	F : Fn(Option<DrawLine<V>>) -> Option<DrawLine<U>>
	{
		DrawLineList(self.0.iter().map(|l| f(l.clone())).collect()) //another questionable clone
	}
}

//would be nicer to move lines out of read_in_lines rather than clone them
pub struct TransformDrawLinesSystem<V : VectorTrait>(pub PhantomData<V>);
impl<'a,V : VectorTrait> System<'a> for TransformDrawLinesSystem<V> {
    type SystemData = (ReadExpect<'a,DrawLineList<V>>,WriteExpect<'a,DrawLineList<V::SubV>>,ReadStorage<'a,Camera<V>>,ReadExpect<'a,Player>);

    fn run(&mut self, (read_in_lines, mut write_out_lines, camera, player) : Self::SystemData) {
    	//write new vec of draw lines to DrawLineList
    	write_out_lines.0 = read_in_lines.0.iter()
    	.map(|line| transform_draw_line(line.clone(),&camera.get(player.0).unwrap()))
    	.collect();

    }
}
//apply transform line to the lines in draw_lines
//need to do nontrivial destructuring and reconstructing
//in order to properly handle Option
//would probably benefit from something monad-like
pub fn transform_draw_line<V : VectorTrait>(
	option_draw_line : Option<DrawLine<V>>,
	camera : &Camera<V>) -> Option<DrawLine<V::SubV>> {
	match option_draw_line {
			Some(draw_line) => {
				let transformed_line = transform_line(Some(draw_line.line),&camera);
				match transformed_line {
					Some(line) => Some(DrawLine{line, color : draw_line.color}),
					None => None
				}
			}
			None => None
		}
}

//in this implementation, the length of the vec is always
//the same, and invisible faces are just sequences of None
//seems to be significantly slower than not padding and just changing the buffer when needed
//either way, we need to modify the method to write to an existing line buffer rather than allocating new Vecs


pub struct VisibilitySystem<V : VectorTrait>(pub PhantomData<V>);

impl<'a,V : VectorTrait> System<'a> for VisibilitySystem<V>  {
	type SystemData = (WriteStorage<'a,Shape<V>>,ReadStorage<'a,Camera<V>>,ReadExpect<'a,Player>,ReadExpect<'a,ClipState<V>>);

	fn run(&mut self, (mut shapes, camera, player, clip_state) : Self::SystemData) {

		for shape in (&mut shapes).join() {

			update_shape_visibility(&camera.get(player.0).unwrap(), shape, &clip_state)
		}
	}

}

//essentially just applies a function to each shape
//replace with a system
pub fn update_shape_visibility<V : VectorTrait>(
	camera : &Camera<V>,
	shape : &mut Shape<V>,
	clip_state : &ClipState<V>
	) {
	//update shape visibility and boundaries
	shape.update_visibility(camera.pos);
	//calculate boundaries for clipping
	if clip_state.clipping_enabled {
		shape.boundaries = crate::clipping::calc_boundaries(
			&shape.faces, &shape.subfaces, camera.pos);
	}

}

pub struct CalcShapesLinesSystem<V : VectorTrait>(pub PhantomData<V>);

impl<'a,V : VectorTrait> System<'a> for CalcShapesLinesSystem<V>  {
	type SystemData = (ReadStorage<'a,Shape<V>>,ReadStorage<'a,ShapeClipState<V>>,
		ReadExpect<'a,Vec<Field>>,ReadExpect<'a,ClipState<V>>,WriteExpect<'a,DrawLineList<V>>);

	fn run(&mut self, (shapes, shape_clip_states, face_scale, clip_state, mut lines) : Self::SystemData) {
			lines.0 = calc_shapes_lines(&shapes, &shape_clip_states, &face_scale, &clip_state);
		}

}

pub fn calc_shapes_lines<V>(
	shapes : &ReadStorage<Shape<V>>,
	shape_clip_states : &ReadStorage<ShapeClipState<V>>,
	face_scale : &Vec<Field>,
	clip_state : &ClipState<V>,
	)  -> Vec<Option<DrawLine<V>>>

where V : VectorTrait
{
	//DEBUG
	// for (i,(sh,s)) in (shapes, shape_clip_states).join().enumerate() {
	// 	println!("shape {}: {}",i,sh.get_pos());
	// 	use itertools::Itertools;
	// 	for e in s.in_front.iter().sorted() {
	// 		print!("{} ",e.id());
	// 	}
	// 	println!("");
	// }
	// panic!();
	//probably worth computing / storing number of lines
	//and using Vec::with_capacity
	let mut lines : Vec<Option<DrawLine<V>>> = Vec::new();
	
	//compute lines for each shape
	for (shape,shape_clip_state) in (shapes,shape_clip_states).join() {
		let mut shape_lines : Vec<Option<DrawLine<V>>> = Vec::new();
		//get lines from each face
		for face in &shape.faces {
			shape_lines.append(&mut face.texture_mapping.draw(face, &shape, &face_scale))
		}

		//clip these lines and append to list
		if clip_state.clipping_enabled {
			let shapes_in_front = shape_clip_state.in_front.iter().map(|&e| shapes.get(e).unwrap());
			let mut clipped_lines = crate::clipping::clip_draw_lines(
				shape_lines,shapes, shapes_in_front);
			lines.append(&mut clipped_lines);
		} else {
			lines.append(&mut shape_lines);
		}
		
	}
	lines
	
}

#[allow(dead_code)]
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

