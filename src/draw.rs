mod texture;
pub use texture::Texture;
pub use texture::TextureMapping;

use crate::camera::{Camera};
use crate::vector::{VectorTrait,Field,VecIndex};
use crate::geometry::{VertIndex,Shape,Line,Face,Edge};
//use crate::graphics;
use crate::clipping::clip_line_plane;
use crate::colors::*;
use crate::clipping::ClipState;

use itertools::Itertools;

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
	let clipped_line : Option<Line<V>> = match line {Some(l) => clip_line_plane(l,&camera.plane,Z_NEAR), None => None};
	let view_line = clipped_line
		.map(|l| l
		.map(|v| view_transform(&camera,v)));
	let proj_line = view_line
		.map(|l| l
		.map(project));
	proj_line
}


pub struct TransformDrawLinesSystem<V : VectorTrait>(V);
impl<'a,V : VectorTrait> System<'a> for TransformDrawLinesSystem<V> {
    type SystemData = (ReadStorage<'a,LineBuffer<V>>,ReadExpect<'a,Camera<V>>);

    fn run(&mut self, (clip_state,shape_data,camera) : Self::SystemData) {
        //(&mut clip_state,shape_data.as_slice(),&camera.pos);
    }
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
				let transformed_line = transform_line(Some(draw_line.line),&camera);
				match transformed_line {
					Some(line) => Some(DrawLine{line, color : draw_line.color}),
					None => None
				}
			}
			None => None
		})
		.collect()
}

// pub fn transform_lines<V>(lines : Vec<Option<Line<V>>>,
// 	camera : ReadExpect<Camera>) -> Vec<Option<Line<V::SubV>>>
// where V : VectorTrait
// {
// 	lines.into_iter().map(|line| transform_line(line,camera)).collect()
	// let clipped_lines = lines
	// .into_iter()
	// .map(|maybe_line| match maybe_line {
	// 	Some(line) => clip_line_plane(line,&camera.plane,Z_NEAR),
	// 	None => None
	// });
 //    //let clipped_lines = lines.map(|line| Some(line)); //no clipping
 //    let view_lines = clipped_lines
 //    	.map(|maybe_line| maybe_line
 //    		.map(|line| line
 //    			.map(|v| view_transform(&camera,v))));
 //    let proj_lines = view_lines
 //    	.map(|maybe_line| maybe_line
 //    		.map(|line| line
 //    			.map(project))).collect();
 //    proj_lines
//}
// pub fn calc_face_lines_new<V : VectorTrait>(
// 	face : &Face<V>, shape : &Shape<V>, face_scales : &Vec<Field>
// ) -> Vec<Option<DrawLine<V>>> {
// 	if face.visible || shape.transparent {
// 		face.
// 	} else {
// 		Vec::new()
// 	}

// }
//in this implementation, the length of the vec is always
//the same, and invisible faces are just sequences of None
//seems to be significantly slower than not padding and just changing the buffer when needed
//either way, we need to modify the method to write to an existing line buffer rather than allocating new Vecs


use specs::{ReadStorage,WriteStorage,ReadExpect,Read,System,Join};

pub struct VisibilitySystem<V : VectorTrait>(pub V);

impl<'a,V : VectorTrait> System<'a> for VisibilitySystem<V>  {
	type SystemData = (WriteStorage<'a,Shape<V>>,ReadExpect<'a,Camera<V>>,ReadExpect<'a,ClipState<V>>);

	fn run(&mut self, (mut shapes, camera, clip_state) : Self::SystemData) {

		for shape in (&mut shapes).join() {

			update_shape_visibility(&camera, shape, &clip_state)
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
pub fn calc_shapes_lines<V>(
	shapes : ReadStorage<Shape<V>>,
	face_scale : &Vec<Field>,
	clip_state : &ClipState<V>,
	)  -> Vec<Option<DrawLine<V>>>

where V : VectorTrait
{
	
	//probably worth computing / storing number of lines
	//and using Vec::with_capacity
	
	let mut lines : Vec<Option<DrawLine<V>>> = Vec::new();
	
	//compute lines for each shape
	for (shape,shape_in_front) in (&shapes).join().zip(clip_state.in_front.iter()) {
		let mut shape_lines : Vec<Option<DrawLine<V>>> = Vec::new();
		//get lines from each face
		for face in &shape.faces {
			shape_lines.append(&mut face.texture_mapping.draw(face, &shape, &face_scale))
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
	shapes : ReadStorage<Shape<V>>,
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
	shapes : ReadStorage<Shape<V>>,
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

