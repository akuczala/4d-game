use crate::vector::{VectorTrait,MatrixTrait,Field,VecIndex,rotation_matrix};
use crate::geometry::{VertIndex,Shape,Line,Plane,Face,Edge};
//use crate::graphics;
use crate::clipping::clip_line_plane;
use crate::colors::*;
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
#[derive(Clone)]
pub enum Texture<V : VectorTrait> {
	DefaultLines{color : Color},
	Lines{lines : Vec<Line<V>>, color : Color},
	DrawLines(Vec<DrawLine<V>>),
}

impl<V: VectorTrait> Texture<V> {
	pub fn set_color(self, color : Color) -> Self {
		match self {
			Texture::DefaultLines{..} => Texture::DefaultLines{color},
			Texture::Lines{lines, ..} => Texture::Lines{lines, color},
			Texture::DrawLines(draw_lines) => Texture::DrawLines(
				draw_lines.into_iter().map(|draw_line| DrawLine{line : draw_line.line,color}).collect()
				),
		}
	}
	pub fn make_tile_texture(scales : &Vec<Field>, n_divisions : &Vec<i32>) -> Texture<V> {
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
		

		let lines = centers.cartesian_product(scales.iter())
			.map(|(center,&scale)| tile_lines.iter()
				.map(|line| line.map(|v| v*scale + center))
				.collect())
			.flat_map(|lines : Vec<Line<V>>| lines)
			.collect();
		Texture::Lines{lines, color : DEFAULT_COLOR}

	}
}

#[derive(Clone)]
pub struct TextureMapping {
	pub frame_vertis : Vec<VertIndex>,
	pub origin_verti : VertIndex
}
impl TextureMapping {
	pub fn draw<V : VectorTrait>(&self, face : &Face<V>, shape : &Shape<V>, face_scales : &Vec<Field>
	) -> Vec<Option<DrawLine<V>>>{
		if !face.visible && !shape.transparent  {
			return Vec::new();
		}
		match &face.texture {
			Texture::DefaultLines{color} => self.draw_default_lines(face,shape,*color,face_scales),
			Texture::Lines{lines,color} => self.draw_lines(shape,lines,*color),
			Texture::DrawLines(draw_lines) => self.draw_drawlines(draw_lines)

		}
		
	}
	pub fn draw_default_lines<V : VectorTrait>(
		&self, face : &Face<V>, shape : &Shape<V>,
		color : Color, face_scales : &Vec<Field>) -> Vec<Option<DrawLine<V>>> {
		let mut lines : Vec<Option<DrawLine<V>>> = Vec::with_capacity(face.edgeis.len()*face_scales.len());
		for &face_scale in face_scales {
			let scale_point = |v| V::linterp(face.center,v,face_scale);
			for edgei in &face.edgeis {
				let edge = &shape.edges[*edgei];
				lines.push(
					Some(DrawLine{
						line : Line(
						shape.verts[edge.0],
						shape.verts[edge.1])
						.map(scale_point),
						color : color
						})
				);
			}
		}
		lines
	}
	pub fn draw_lines<V : VectorTrait>(&self, shape : &Shape<V>,
		lines : &Vec<Line<V::SubV>>, color : Color) -> Vec<Option<DrawLine<V>>> {
		let origin = shape.verts[self.origin_verti];
		let frame_verts : Vec<V> = self.frame_vertis.iter().map(|&vi| shape.verts[vi] - origin).collect();
		//this is pretty ridiculous. it just matrix multiplies a matrix of frame_verts (as columns) by each vertex
		//in every line, then adds the origin.
		lines.iter().map(|line|
			line.map(|v| (0..V::SubV::DIM).zip(frame_verts.iter()).map(|(i,&f)| f*v[i]).fold(V::zero(),|a,b| a + b) + origin)
			).map(|line| Some(DrawLine{line,color})).collect()
	}
	pub fn draw_drawlines<V : VectorTrait>(&self, draw_lines : &Vec<DrawLine<V::SubV>>) -> Vec<Option<DrawLine<V>>> {
		Vec::new()
		//draw_lines.iter().map(|draw_line| Some(draw_line.clone())).collect()
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
pub fn calc_face_lines_old<V : VectorTrait>(
	face : &Face<V>,
	shape : &Shape<V>,
	face_scales : &Vec<Field>
) -> Vec<Option<DrawLine<V>>> {
	if face.visible || shape.transparent {
		let mut lines : Vec<Option<DrawLine<V>>> = Vec::with_capacity(face.edgeis.len()*face_scales.len());
		for &face_scale in face_scales {
			let scale_point = |v| V::linterp(face.center,v,face_scale);
			for edgei in &face.edgeis {
				let edge = &shape.edges[*edgei];
				//println!("{}",edge);
				lines.push(
					Some(DrawLine{
						line : Line(
						shape.verts[edge.0],
						shape.verts[edge.1])
						.map(scale_point),
						color : DEFAULT_COLOR
						})
				);
			}
		}
		lines
	} else {
		vec![None ; face.edgeis.len()*face_scales.len()]
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
			//shape_lines.append(&mut calc_face_lines_old(face,&shape,&face_scale));
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

