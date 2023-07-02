use core::panic;

use itertools::Itertools;

use crate::{vector::{VectorTrait, Field, linspace, VecIndex}, geometry::{Line, shape::{VertIndex, buildshapes::{ShapeBuilder, convex_shape_to_face_shape, build_prism_2d}}}, graphics::colors::{Color, MAGENTA, RED, GREEN, CYAN, blend, BLUE}, components::{Shape, Transform, Transformable}, constants::{HALF_PI, CARDINAL_COLORS, ZERO, HALF}};

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

pub fn calc_wireframe_lines<V: VectorTrait>(shape : &Shape<V>) -> Vec<Line<V>>
{
	//concatenate vertex indices from each edge to get list
	//of indices for drawing lines
	let mut vertis : Vec<VertIndex> = Vec::new(); 
    for edge in shape.edges.iter() {
        vertis.push(edge.0);
        vertis.push(edge.1);
    }

    vertis.chunks(2)
    	.map(|pair| Line(shape.verts[pair[0]],shape.verts[pair[1]])).collect()


}
pub fn calc_normals_lines<V: VectorTrait>(shape : &Shape<V>) -> Vec<Line<V>>
{
	shape.faces.iter().map(|face| Line(
			face.center(), face.center() + face.normal()/2.0
		)
	).collect()

}

pub fn draw_axes<'a, V: VectorTrait + 'a>(center: V, len: Field) -> impl Iterator<Item = DrawLine<V>> {
	(0..V::DIM)
		.zip([RED, GREEN, CYAN, MAGENTA])
		.map(
			move |(i, color)| DrawLine{
				line: Line(
					center - V::one_hot(i) * len,
					center + V::one_hot(i) * len
				),
				color
			}
		)
}

pub fn draw_stars<V: VectorTrait>() -> Vec<DrawLine<V>>{
	iproduct!((0..V::DIM), [false, true])
		.zip(CARDINAL_COLORS)
		.flat_map(
			|((axis, sign), color)| draw_star(axis, sign).into_iter()
				.map(move |line| DrawLine{line, color})
		)
		.collect_vec()
}

fn draw_star<V: VectorTrait>(axis: VecIndex, sign: bool) -> Vec<Line<V>> {
	let sub_cube: Shape<V::SubV> = ShapeBuilder::build_cube(1e3).build();
	let (mut cube, _) = convex_shape_to_face_shape::<V>(sub_cube, true);
	cube.update_from_ref(
		&cube.clone(),
		&Transform::identity()
			.with_rotation(-1, axis, if axis != V::DIM {HALF_PI} else {ZERO})
			.with_translation(V::one_hot(axis) * if sign {1.0} else {-1.0} * 1e5)
	);
	calc_wireframe_lines(&cube)

}

pub fn draw_horizon<V: VectorTrait>() -> Vec<Line<V>> {
	calc_wireframe_lines(&(
		match V::DIM {
			3 => ShapeBuilder::build_prism(2, &vec![1e5], &vec![12])
				.with_rotation(-1, 1, HALF_PI)
				.build(),
			4 => todo!(),
			_ => panic!("draw_horizon not supported in {} dim", V::DIM)
		}
	))
}

pub fn draw_sky<V: VectorTrait>(n_pts: usize) -> Vec<DrawLine<V>> {
	(0..n_pts).map(
		|_| {
			let pos = random_hemisphere_point(V::one_hot(1)) * 1e4;
			DrawLine{
				line: pointlike_sky_line(pos),
				color: blend(CYAN, BLUE, pos.normalize().dot(V::one_hot(1)))
			}
		}
	).collect_vec()
}

fn random_sphere_point<V: VectorTrait>() -> V {
	(V::random() - V::ones() * HALF).normalize()
}

fn random_hemisphere_point<V: VectorTrait>(normal: V) -> V {
	let v: V = random_sphere_point();
	if v.dot(normal) > ZERO {v} else {-v}
}

fn pointlike_sky_line<V: VectorTrait>(pos: V) -> Line<V> {
	Line(pos, pos + random_sphere_point::<V>() * 100.0)
}