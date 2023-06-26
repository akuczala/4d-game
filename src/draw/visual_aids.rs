use crate::{vector::{VectorTrait, Field, linspace}, geometry::{Line, shape::VertIndex}, graphics::colors::{Color, MAGENTA, RED, GREEN, CYAN}, components::Shape};

use super::DrawLine;

pub fn calc_grid_lines<V: VectorTrait>(center: V, cell_size: Field, n: usize) -> Vec<Line<V>> {
	let axes = (0, 2, 3);
	
	let deltas: Vec<Field> = linspace(-cell_size * (n as Field), cell_size * (n as Field), 2 * n + 1).collect();
	let (si, sf) = (deltas[0], deltas[deltas.len() - 1]);
	fn make_line<V: VectorTrait>(center: V, unit_1: V, unit_2: V, si: Field, sf: Field, s: Field) -> Line<V> {
		Line(unit_1 * s + unit_2 * si + center, unit_1 * s + unit_2 * sf + center)
	}
	let x_lines = deltas.iter()
		.map(|&s| make_line(center, V::one_hot(axes.0), V::one_hot(axes.1), si, sf, s));
	let z_lines = deltas.iter()
		.map(|&s| make_line(center, V::one_hot(axes.1), V::one_hot(axes.0), si, sf, s));
	
	let xz_lines = x_lines.chain(z_lines).collect();
	match V::DIM {
		3 => xz_lines,
		4 => {
			let xz_planes = deltas.iter()
				.map(
					|&s| xz_lines.iter()
					.map(
						move |line| line.map(|p| p + V::one_hot(3) * s)
					)
				)
				.flat_map(|iter| iter);
			let w_lines = iproduct!(deltas.iter(), deltas.iter())
				.map(
					|(&x, &z)| Line(
						V::one_hot(3) * si, V::one_hot(3) * sf
					).map(
						|p| p +V::one_hot(0) * x + V::one_hot(2) * z + center
					)
				);
			xz_planes.chain(w_lines).collect()
		},
		i => panic!("Unsupported dimension {} for calc_grid_lines", i)
	}

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
pub fn draw_wireframe_with_normals<V: VectorTrait>(
	shape : &Shape<V>, color : Color) -> Vec<Option<DrawLine<V>>>
{
	let mut draw_lines = draw_wireframe(shape, color);
	draw_lines.extend(
		shape.faces.iter().map(|face|
			Some(DrawLine{
				line: Line(face.center(), face.center() + face.normal()/2.0),
				color
			})
		)
	);
	draw_lines

}

pub fn draw_axes<V: VectorTrait>(center: V, len: Field) -> Vec<DrawLine<V>> {
	(0..V::DIM)
		.zip([RED, GREEN, CYAN, MAGENTA])
		.map(
			|(i, color)| DrawLine{
				line: Line(
					center,
					center + V::one_hot(i) * len
				),
				color
			}
		).collect()
}