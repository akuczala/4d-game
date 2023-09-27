use itertools::Itertools;

use crate::{
    components::{Shape, Transform, Transformable},
    config::CompassConfig,
    constants::{
        AXES_COLORS, CARDINAL_COLORS, HALF_PI, SKY_DISTANCE, SKY_FUZZ_SIZE, STAR_SIZE, UP_AXIS,
        ZERO,
    },
    geometry::{
        shape::{
            buildshapes::{convex_shape_to_face_shape, ShapeBuilder},
            VertIndex,
        },
        transform::Scaling,
        Line,
    },
    graphics::colors::{blend, Color, BLUE, CYAN},
    utils::ValidDimension,
    vector::{linspace, random_sphere_point, Field, VecIndex, VectorTrait},
};

use super::DrawLine;

pub fn calc_grid_lines<V: VectorTrait>(center: V, cell_size: Field, n: usize) -> Vec<Line<V>> {
    let axes = (0, 2, 3);

    let deltas: Vec<Field> = linspace(
        -cell_size * (n as Field),
        cell_size * (n as Field),
        2 * n + 1,
    )
    .collect();
    let (si, sf) = (deltas[0], deltas[deltas.len() - 1]);
    fn make_line<V: VectorTrait>(
        center: V,
        unit_1: V,
        unit_2: V,
        si: Field,
        sf: Field,
        s: Field,
    ) -> Line<V> {
        Line(
            unit_1 * s + unit_2 * si + center,
            unit_1 * s + unit_2 * sf + center,
        )
    }
    let x_lines = deltas
        .iter()
        .map(|&s| make_line(center, V::one_hot(axes.0), V::one_hot(axes.1), si, sf, s));
    let z_lines = deltas
        .iter()
        .map(|&s| make_line(center, V::one_hot(axes.1), V::one_hot(axes.0), si, sf, s));

    let xz_lines = x_lines.chain(z_lines).collect();
    match V::DIM.into() {
        ValidDimension::Three => xz_lines,
        ValidDimension::Four => {
            let xz_planes = deltas.iter().flat_map(|&s| {
                xz_lines
                    .iter()
                    .map(move |line| line.map(|p| p + V::one_hot(3) * s))
            });
            let w_lines = iproduct!(deltas.iter(), deltas.iter()).map(|(&x, &z)| {
                Line(V::one_hot(3) * si, V::one_hot(3) * sf)
                    .map(|p| p + V::one_hot(0) * x + V::one_hot(2) * z + center)
            });
            xz_planes.chain(w_lines).collect()
        }
    }
}

pub fn calc_wireframe_lines<V: VectorTrait>(shape: &Shape<V>) -> Vec<Line<V>> {
    //concatenate vertex indices from each edge to get list
    //of indices for drawing lines
    let mut vertis: Vec<VertIndex> = Vec::new();
    for edge in shape.edges.iter() {
        vertis.push(edge.0);
        vertis.push(edge.1);
    }

    vertis
        .chunks(2)
        .map(|pair| Line(shape.verts[pair[0]], shape.verts[pair[1]]))
        .collect()
}

#[allow(dead_code)]
pub fn calc_normals_lines<V: VectorTrait>(shape: &Shape<V>) -> Vec<Line<V>> {
    shape
        .faces
        .iter()
        .map(|face| Line(face.center(), face.center() + face.normal() / 2.0))
        .collect()
}

pub fn draw_axes<'a, V: VectorTrait + 'a>(
    center: V,
    len: Field,
) -> impl Iterator<Item = DrawLine<V>> {
    (0..V::DIM)
        .zip(AXES_COLORS)
        .map(move |(i, color)| DrawLine {
            line: Line(center - V::one_hot(i) * len, center + V::one_hot(i) * len),
            color,
        })
}

pub fn draw_stars<V: VectorTrait>() -> Vec<DrawLine<V>> {
    iproduct!((0..V::DIM), [false, true])
        .zip(CARDINAL_COLORS)
        .flat_map(|((axis, sign), color)| {
            draw_star(axis, sign)
                .into_iter()
                .map(move |line| DrawLine { line, color })
        })
        .collect_vec()
}

fn draw_star<V: VectorTrait>(axis: VecIndex, sign: bool) -> Vec<Line<V>> {
    let sub_cube: Shape<V::SubV> = ShapeBuilder::build_cube(STAR_SIZE).build();
    let mut cube = convex_shape_to_face_shape::<V>(sub_cube, true);
    cube.update_from_ref(
        &cube.clone(),
        &Transform::identity()
            .with_rotation(-1, axis, if axis != V::DIM { HALF_PI } else { ZERO })
            .with_translation(V::one_hot(axis) * if sign { 1.0 } else { -1.0 } * SKY_DISTANCE),
    );
    calc_wireframe_lines(&cube)
}

pub fn draw_horizon<V: VectorTrait>(n_lines: usize) -> Vec<Line<V>> {
    match V::DIM.into() {
        ValidDimension::Three => calc_wireframe_lines(
            &(ShapeBuilder::build_prism(2, &[SKY_DISTANCE], &[12]))
                .with_rotation(-1, 1, HALF_PI)
                .build(),
        ),
        ValidDimension::Four => (0..n_lines)
            .map(|_| {
                pointlike_sky_line({
                    let u = random_sphere_point::<V::SubV>() * SKY_DISTANCE;
                    [u[0], ZERO, u[1], u[2]].into_iter().collect()
                })
            })
            .collect(),
    }
}

pub fn draw_sky<V: VectorTrait>(n_lines: usize) -> Vec<DrawLine<V>> {
    (0..n_lines)
        .map(|_| {
            let pos = random_hemisphere_point(V::one_hot(UP_AXIS)) * SKY_DISTANCE;
            DrawLine {
                line: pointlike_sky_line(pos),
                color: blend(CYAN, BLUE, pos.normalize().dot(V::one_hot(UP_AXIS))),
            }
        })
        .collect_vec()
}

fn random_hemisphere_point<V: VectorTrait>(normal: V) -> V {
    let v: V = random_sphere_point();
    if v.dot(normal) > ZERO {
        v
    } else {
        -v
    }
}

fn pointlike_sky_line<V: VectorTrait>(pos: V) -> Line<V> {
    Line(pos, pos + random_sphere_point::<V>() * SKY_FUZZ_SIZE)
}

pub(super) enum CompassIcon {
    Cube,
    Hex,
}
pub(super) struct CompassPoint<V: VectorTrait> {
    pub point: V,
    pub icon: CompassIcon,
    pub color: Color,
}

pub(super) fn draw_compass<V: VectorTrait, I: Iterator<Item = CompassPoint<V>>>(
    config: &CompassConfig,
    heading_matrix: V::M,
    compass_points: I,
    draw_line_list: &mut Vec<DrawLine<V::SubV>>,
) {
    // project unit vectors pointing at horizon into (D - 2) sphere, with heading vec at "center"
    // TODO: store lines and apply heading transform
    //     : add declination indicator
    //     : pipe into separate 2d gui render pipeline?
    //     : alpha by distance
    // draw a circle?

    let center =
        V::SubV::one_hot(0) * config.center[0] + V::SubV::one_hot(UP_AXIS) * config.center[1];
    let get_alpha = |x: Field| (x + 1.0) / 2.0;
    // forward direction is centermost
    let rotation = heading_matrix;

    let cardinal_points = iproduct!((0..V::DIM), [-1.0, 1.0])
        .zip(CARDINAL_COLORS)
        .filter(|t| t.0 .0 != UP_AXIS)
        .map(|((i, s), color)| CompassPoint {
            point: V::one_hot(i) * s,
            icon: CompassIcon::Cube,
            color,
        });

    let sub_cube_lines: Vec<Line<V::SubV>> =
        calc_wireframe_lines(&ShapeBuilder::build_cube(config.icon_size).build());
    let hex_lines: Vec<Line<V::SubV>> = calc_wireframe_lines(
        &ShapeBuilder::build_coin()
            .with_scale(Scaling::Scalar(config.icon_size * 8.0))
            .build(),
    );
    draw_line_list.extend(
        cardinal_points
            .chain(compass_points.into_iter())
            .flat_map(|cp| {
                let center_point = rotation * cp.point;
                let alpha = get_alpha(center_point[-1]);
                let proj_center_point = V::project(&(center_point * config.radius)) + center;
                (match cp.icon {
                    CompassIcon::Cube => sub_cube_lines.iter(),
                    CompassIcon::Hex => hex_lines.iter(),
                })
                .map(move |line| DrawLine {
                    line: line.map(|p| p + proj_center_point),
                    color: cp.color.with_alpha(alpha),
                })
            }),
    );
}
