use std::marker::PhantomData;

use itertools::Itertools;
use rand::seq::IteratorRandom;
use serde::__private::de;
use specs::prelude::*;

use clipping::{ClipState,clip_line_plane, clip_line_cube};
use specs::rayon::iter::Chain;
pub use texture::{Texture, TextureMapping, ShapeTexture, FaceTexture};

use crate::components::*;
use crate::ecs_utils::Componentable;
use crate::geometry::Face;
use crate::geometry::{Line, Shape, shape::VertIndex};
use crate::graphics::colors::*;
use crate::vector::{Field, VectorTrait, linspace, VecIndex, barycenter};

use self::clipping::{clip_line_cylinder, clip_line_sphere, clip_line_tube};
use self::texture::draw_face_texture;
use self::visual_aids::{draw_wireframe, draw_axes};

pub mod texture;
pub mod clipping;
pub mod draw_line_collection;
pub mod visual_aids;

extern crate map_in_place;

const Z0 : Field = 0.0;

const SMALL_Z : Field = 0.001;
const Z_NEAR : Field = 0.1; 

const CLIP_SPHERE_RADIUS : Field = 0.5;

const VIEWPORT_SHAPE: ViewportShape = ViewportShape::Cylinder;

#[derive(Clone, Copy)]
pub enum ViewportShape {
	Cube,
	Sphere,
	Cylinder,
	Tube,
	None
}

#[derive(Clone,Copy)]
pub struct DrawVertex<V>
where V: VectorTrait
{
	pub vertex : V,
	pub color : Color
}

#[derive(Clone)]
pub struct DrawLine<V>
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
fn view_transform<V: VectorTrait>(transform : &Transform<V, V::M>, point : V) -> V
{
	transform.frame * (point - transform.pos)
}
//this takes a Option<Line> and returns Option<Line>
//can likely remove camera here by calculating the plane from the transform, unless you want the
//camera's plane to differ from its position/heading
pub fn transform_line<V: VectorTrait>(line : Option<Line<V>>, transform : &Transform<V, V::M>, camera: &Camera<V, V::M>) -> Option<Line<V::SubV>>
where V : VectorTrait
{
	let clipped_line = line.and_then(|l| clip_line_plane(l,&camera.plane,Z_NEAR));

	let view_line = clipped_line
		.map(|l| l
		.map(|v| view_transform(transform,v)));
	let proj_line = view_line
		.map(|l| l
		.map(project));
	let clip_proj_line = proj_line.and_then(
		|l| match VIEWPORT_SHAPE {
			ViewportShape::Cube => clip_line_cube(l,CLIP_SPHERE_RADIUS),
			ViewportShape::Sphere => clip_line_sphere(l,CLIP_SPHERE_RADIUS),
			ViewportShape::Cylinder => clip_line_cylinder(l,CLIP_SPHERE_RADIUS, CLIP_SPHERE_RADIUS),
			ViewportShape::Tube => clip_line_tube(l, CLIP_SPHERE_RADIUS),
			ViewportShape::None => Some(l),
		} 
	);
	clip_proj_line
}

pub struct DrawLineList<V>(pub Vec<Option<DrawLine<V>>>);
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
pub struct TransformDrawLinesSystem<V>(pub PhantomData<V>);
impl<'a, V, U, M> System<'a> for TransformDrawLinesSystem<V>
where
	V: VectorTrait<M = M, SubV = U> + Componentable,
	U: Componentable,
	M: Componentable,
{
    type SystemData = (
		ReadExpect<'a,DrawLineList<V>>,
		WriteExpect<'a,DrawLineList<V::SubV>>,
		ReadStorage<'a,Camera<V, V::M>>,
		ReadStorage<'a,Transform<V, V::M>>,
		ReadExpect<'a,Player>);

    fn run(&mut self, (read_in_lines, mut write_out_lines, camera, transform, player) : Self::SystemData) {
    	//write new vec of draw lines to DrawLineList
    	write_out_lines.0 = read_in_lines.0.iter()
    	.map(|line| transform_draw_line(line.clone(),&transform.get(player.0).unwrap(), &camera.get(player.0).unwrap()))
    	.collect();

    }
}
//apply transform line to the lines in draw_lines
//need to do nontrivial destructuring and reconstructing
//in order to properly handle Option
//would probably benefit from something monad-like
pub fn transform_draw_line<V : VectorTrait>(
	option_draw_line : Option<DrawLine<V>>,
	transform: &Transform<V, V::M>,
	camera : &Camera<V, V::M>) -> Option<DrawLine<V::SubV>> {
	match option_draw_line {
			Some(draw_line) => {
				let transformed_line = transform_line(Some(draw_line.line),&transform,&camera);
				match transformed_line {
					Some(line) => Some(DrawLine{line, color : draw_line.color}),
					None => None
				}
			}
			None => None
		}
}

// pub struct DrawTargetSystem<V : VectorTrait>(pub PhantomData<V>);
// impl<'a,V : VectorTrait> System<'a> for DrawTargetSystem<V> {
//     type SystemData = (ReadExpect<'a,Player>,ReadStorage<'a,MaybeTarget<V>>,WriteExpect<'a,DrawLineList<V>>);

//     fn run(&mut self, (player, maybe_target, mut draw_lines) : Self::SystemData) {
//     	//write new vec of draw lines to DrawLineList
//     	if let Some(target) = maybe_target.get(player.0).unwrap().0 {
// 	    		for line in draw_wireframe(&crate::geometry::buildshapes::build_cube_3d(0.04),WHITE).into_iter() {
// 	    			draw_lines.0.push(line);
// 	    		}
    	
//     	}
    	
//     }
// }

pub struct DrawCursorSystem<V>(pub PhantomData<V>);
impl<'a, V, U> System<'a> for DrawCursorSystem<V>
where
	V: VectorTrait<SubV=U> + Componentable,
	U: VectorTrait + Componentable
{
    type SystemData = (
		ReadStorage<'a,Cursor>,
		ReadStorage<'a,Shape<U>>,
		WriteExpect<'a,DrawLineList<U>>
	);

    fn run(&mut self, (cursors, shapes, mut draw_lines) : Self::SystemData) {
    	//write new vec of draw lines to DrawLineList
    	for (_,shape) in (&cursors,&shapes).join() {
    		for line in draw_wireframe(shape,WHITE).into_iter() {
    			draw_lines.0.push(line);
    		}
    	}
    }
}

pub struct DrawSelectionBox<V>(pub PhantomData<V>);
impl<'a,V : VectorTrait + Componentable> System<'a> for DrawSelectionBox<V>
{
	type SystemData = (
		ReadStorage<'a,MaybeSelected<V>>,
		WriteExpect<'a,DrawLineList<V>>
	);

	fn run(&mut self, (selected_storage, mut draw_lines) : Self::SystemData) {
		//write new vec of draw lines to DrawLineList

		for maybe_selected in (&selected_storage).join() {
			if let MaybeSelected(Some(selected)) = maybe_selected {
				draw_lines.0.extend(
					draw_wireframe(&selected.selection_box_shape,WHITE)
				);
				draw_lines.0.extend(
					draw_axes(barycenter(&selected.selection_box_shape.verts), 1.0)
					.into_iter()
					.map(Option::Some)
				)
			}

		}
	}
}

//in this implementation, the length of the vec is always
//the same, and invisible faces are just sequences of None
//seems to be significantly slower than not padding and just changing the buffer when needed
//either way, we need to modify the method to write to an existing line buffer rather than allocating new Vecs


pub struct VisibilitySystem<V>(pub PhantomData<V>);

impl<'a, V, M> System<'a> for VisibilitySystem<V>
where
	V: VectorTrait<M=M> + Componentable,
	M: Componentable
{
	type SystemData = (
		ReadStorage<'a,Shape<V>>,
		WriteStorage<'a,ShapeClipState<V>>,
		ReadStorage<'a,ShapeType<V>>,
		ReadStorage<'a,Transform<V, V::M>>,
		ReadExpect<'a,Player>,
		ReadExpect<'a,ClipState<V>>
	);

	fn run(
		&mut self, (
			shapes,
			mut shape_clip_states,
			shape_types,
			transform,
			player,
			clip_state
		) : Self::SystemData) {

		for (shape,shape_clip_state, shape_type) in (&shapes, &mut shape_clip_states, &shape_types).join() {

			update_shape_visibility(transform.get(player.0).unwrap().pos, shape, shape_clip_state, shape_type, &clip_state)
		}
	}

}

//updates clipping boundaries and face visibility based on normals
// mutated: shape_clip_state boundaries and face_visibility
pub fn update_shape_visibility<V : VectorTrait>(
	camera_pos: V,
	shape: &Shape<V>,
	shape_clip_state : &mut ShapeClipState<V>,
	shape_type: &ShapeType<V>,
	clip_state: &ClipState<V>
	) {
	//update shape visibility and boundaries
	let two_sided = match shape_type {
		ShapeType::Convex(_) => false,
		ShapeType::SingleFace(single_face) => single_face.two_sided
	};
	// build face visibility vec if empty
	if shape_clip_state.face_visibility.is_empty() {
		for face in shape.faces.iter() {
			shape_clip_state.face_visibility.push(get_face_visibility::<V>(face, camera_pos,shape_clip_state.transparent | two_sided));
		}
	} else {
		for (face, visible) in shape.faces.iter().zip(shape_clip_state.face_visibility.iter_mut()) {
			*visible = get_face_visibility(face, camera_pos, shape_clip_state.transparent | two_sided);
		}
	}
	
	//calculate boundaries for clipping
	if clip_state.clipping_enabled {
		shape_clip_state.boundaries = match shape_type {
			ShapeType::Convex(convex) => convex.calc_boundaries(camera_pos, &shape.faces, &mut shape_clip_state.face_visibility),
			ShapeType::SingleFace(single_face) => single_face.calc_boundaries(
				camera_pos, &shape.verts, shape.faces[0].center(), shape_clip_state.face_visibility[0]
			),
		};
	}

}

pub fn get_face_visibility<V: VectorTrait>(face: &Face<V>,camera_pos : V, two_sided: bool) -> bool
    {
        return two_sided | (
            face.plane().point_signed_distance(camera_pos) > 0.0
        )
    }

pub struct CalcShapesLinesSystem<V>(pub PhantomData<V>);

impl<'a, V, U, M> System<'a> for CalcShapesLinesSystem<V> 
where
	V: VectorTrait<M = M, SubV =U> + Componentable,
	U: VectorTrait + Componentable,
	M: Componentable
{
	type SystemData = (
		ReadStorage<'a,Shape<V>>,
		ReadStorage<'a, ShapeTexture<U>>,
		ReadStorage<'a, ShapeClipState<V>>,
		ReadExpect<'a, Vec<Field>>,
		ReadExpect<'a, ClipState<V>>,
		WriteExpect<'a, DrawLineList<V>> // TODO: break up into components so that these can be processed more in parallel with par_iter?
	);

	fn run(&mut self, (
		shapes,
		shape_textures,
		shape_clip_states,
		face_scale,
		clip_state,
		mut lines
	) : Self::SystemData) {
			lines.0 = calc_shapes_lines(
				&shapes,
				&shape_textures,
				&shape_clip_states,
				&face_scale,
				&clip_state
			);
		}

}

pub fn calc_shapes_lines<V, U, M>(
	shapes : &ReadStorage<Shape<V>>,
	shape_textures: &ReadStorage<ShapeTexture<U>>,
	shape_clip_states : &ReadStorage<ShapeClipState<V>>,
	face_scale : &Vec<Field>,
	clip_state : &ClipState<V>,
	)  -> Vec<Option<DrawLine<V>>>
where
	V: VectorTrait<M = M, SubV =U> + Componentable,
	U: VectorTrait + Componentable,
	M: Componentable
{
	//DEBUG: list entities in front of each shape
	// for (i,(sh,s)) in (shapes, shape_clip_states).join().enumerate() {
	// 	println!("shape {}: {}",i,sh.get_pos());
	// 	println!("{}",s.in_front_debug());
	// }
	// panic!();
	//probably worth computing / storing number of lines
	//and using Vec::with_capacity
	let mut lines : Vec<Option<DrawLine<V>>> = Vec::new();
	
	//compute lines for each shape
	for (shape, shape_texture, shape_clip_state) in (shapes, shape_textures, shape_clip_states).join() {
		let mut shape_lines : Vec<Option<DrawLine<V>>> = Vec::new();
		//get lines from each face
		for (face, &visible, face_texture) in izip!(shape.faces.iter(), shape_clip_state.face_visibility.iter(), shape_texture.face_textures.iter()) {
			shape_lines.append(&mut draw_face_texture::<V>(&face_texture, face, &shape, &face_scale, visible))
		}

		//clip these lines and append to list
		if clip_state.clipping_enabled {
			let clip_states_in_front = shape_clip_state.in_front.iter()
				.map(|&e| match shape_clip_states.get(e) {
					Some(s) => s,
					None => panic!("Invalid entity {} found in shape_clip_state",e.id()),
				});
			//do clipping between all shapes
			//let shapes_in_front = shapes.join().filter(|&s| (s as *const _ ) != (shape as *const _));
			let mut clipped_lines = clipping::clip_draw_lines(
				shape_lines, clip_states_in_front);
			lines.append(&mut clipped_lines);
		} else {
			lines.append(&mut shape_lines);
		}
		
	}
	lines
	
}
