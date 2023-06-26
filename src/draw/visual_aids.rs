use crate::{vector::{VectorTrait, Field, linspace}, geometry::Line};

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