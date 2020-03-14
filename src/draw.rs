pub mod camera;
pub mod texture;
pub mod line_buffer;

pub use line_buffer::{Buffer};
pub use camera::Camera;
pub use texture::{Texture,TextureMapping};

use crate::vector::{VectorTrait,Field};
use crate::geometry::{VertIndex,Shape,Line,Face};
//use crate::graphics;
use crate::clipping::clip_line_plane;
use crate::colors::*;
use crate::clipping::ClipState;

const Z0 : Field = 0.0;

const SMALL_Z : Field = 0.001;
const Z_NEAR : Field = 0.1; 

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
pub trait LineGenerator<V : VectorTrait>: Iterator<Item=Line<V>> {}
pub trait DrawLineGenerator<V : VectorTrait>: Iterator<Item=DrawLine<V>> {}

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
pub fn transform_draw_line<V : VectorTrait>(
	option_draw_line : Option<DrawLine<V>>,
	camera : &Camera<V>) -> Option<DrawLine<V::SubV>> {
	match option_draw_line {
			Some(draw_line) => {
				let transformed_line = transform_line(draw_line.line,&camera);
				match transformed_line {
					Some(line) => Some(DrawLine{line, color : draw_line.color}),
					None => None
				}
			}
			None => None
		}
}
// pub fn transform_draw_lines<V : VectorTrait>(
// 	draw_lines : Vec<Option<DrawLine<V>>>,
// 	camera : &Camera<V>) -> Vec<Option<DrawLine<V::SubV>>> {
// 	draw_lines.into_iter()
// 		.map(|opt_draw_line| transform_draw_line(opt_draw_line,camera))
// 		.collect()
// }
pub fn transform_draw_lines<V : VectorTrait>(
	in_buffer : &mut Buffer<Option<DrawLine<V>>>,
	out_buffer : &mut Buffer<Option<DrawLine<V::SubV>>>,
	camera : &Camera<V>) {

	out_buffer.clear();
	in_buffer.to_beginning();

	for i in 0..in_buffer.cur_size {
		out_buffer.add(
			transform_draw_line(in_buffer.get_ref(i).clone(),camera)
			);
	}
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


//want to do, for each line in Texture for each face,
//Texture -> Texture Mapping -> Clipping -> projection -> (Viewport Clipping?) -> buffer

//can alternatively do
//Texture -> Texture Mapping -> Clipping
//and then map projection over Line Buffer
//if we don't do viewport clipping, or if it can be done in the full dimensional space,
//we could probably speed things up by doing projection on the GPU
//since it is a pointwise operation
//on the other hand, projection is probably really cheap anyways

//in an ideal world, we could write this whole pipeline as a really fancy set_from call
//on the line buffer (if it was an iterator, which we may as well make it)

//in principle, we could have a hierarchy of iterators that yield lines, for example
//Texture has an iterator that yields its lines
//TextureMapping has an iterator that takes lines from Texture and maps them to face
//each face has an iterator that yields lines from its texture mapping
//each shape has an iterator that yields lines from each of its faces
//...hopefully this would be as efficient as a big for loop that processes each line
//and hopefully this approach would be far more elegant than doing that explicitly
//we would need a nice way to terminate and reset each iterator for each draw command
//oh apparently this is just done by returning None

pub fn calc_shapes_lines<V : VectorTrait>(
	output_buffer :  &mut Buffer<Option<DrawLine<V>>>,
	shape_buffer : &mut Buffer<Option<DrawLine<V>>>,
	extra_buffer : &mut Buffer<Option<DrawLine<V>>>,
	shapes : &mut Vec<Shape<V>>,
	face_scale : &Vec<Field>,
	clip_state : &ClipState<V>,
	)
{
	
	//compute lines for each shape
	for (shape,shape_in_front) in shapes.iter().zip(clip_state.in_front.iter()) {
		shape_buffer.clear();
		//get lines from each face
		for face in &shape.faces {
			
			face.texture_mapping.draw(shape_buffer,face, &shape, &face_scale)
		}
		extra_buffer.clear();
		//clip these lines and append to list
		if clip_state.clipping_enabled {
			crate::clipping::clip_draw_lines(shape_buffer,
				extra_buffer, shapes, Some(shape_in_front));
		}
		shape_buffer.copy_to_buffer(output_buffer);
		
	}
	
}
// pub fn calc_lines_color<V : VectorTrait>(
// 	shapes : &Vec<Shape<V>>,
// 	lines : Vec<Line<V>>,
// 	color : Color
// 	) -> Vec<Option<DrawLine<V>>> {

// 	let draw_lines = lines
// 		.into_iter()
// 		.map(|line| Some(DrawLine{line : line,color}))
// 		.collect();

// 	let clipped_lines = crate::clipping::clip_draw_lines(
// 			draw_lines, shapes, None);

// 	clipped_lines
// }
//ehh. need to clone in here since we're borrowing lines
// pub fn calc_lines_color_from_ref<V : VectorTrait>(
// 	shapes : &Vec<Shape<V>>,
// 	lines : &Vec<Line<V>>,
// 	color : Color
// 	) -> Vec<Option<DrawLine<V>>> {

// 	let draw_lines = lines
// 		.iter()
// 		.map(|line| Some(DrawLine{line : (*line).clone(),color}))
// 		.collect();

// 	let clipped_lines = crate::clipping::clip_draw_lines(
// 			draw_lines, shapes, None);

// 	clipped_lines
// }

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

