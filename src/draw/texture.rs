use super::{DrawLine};

use crate::vector::{VectorTrait,Field,VecIndex};
use crate::geometry::{VertIndex,Shape,Line,Face,Edge};

use crate::colors::*;


use itertools::Itertools;

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
		if !face.visible {
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
		unimplemented!()
		//draw_lines.iter().map(|draw_line| Some(draw_line.clone())).collect()
	}
	//use face edges and reference vertices to determine vertex indices for texture mapping
	//order by side length, in decreasing order
	pub fn calc_cube_vertis<V : VectorTrait>(face : &Face<V>, verts : &Vec<V>, edges : &Vec<Edge>) -> Self {
		let face_vertis = & face.vertis;
		let origin_verti = face_vertis[0]; //arbitrary
		//get list of vertis connected by an edge to origin verti
		let frame_vertis = face.edgeis.iter().map(|&ei| &edges[ei])
			.filter_map(|edge| {
				match edge {
					Edge(v1,v2) if *v1 == origin_verti => Some(*v2),
					Edge(v1,v2) if *v2 == origin_verti => Some(*v1),
					_ => None
				}
			});
		let sorted_frame_vertis : Vec<VertIndex> = frame_vertis
			.map(|vi| (vi,(verts[vi]-verts[origin_verti]).norm()))
			.sorted_by(|a,b| b.1.partial_cmp(&a.1).unwrap())
			.map(|(vi,_v)| vi)
			.collect();
		// for &vi in &sorted_frame_vertis {
		// 	println!("{}",(verts[vi]-verts[origin_verti]).norm() );
		// }
		TextureMapping{origin_verti,frame_vertis : sorted_frame_vertis}
	}
}