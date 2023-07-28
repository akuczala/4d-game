use colored::Colorize;

use crate::geometry::shape::single_face::{make_3d_square, make_3d_triangle};
use crate::geometry::shape::Edge;
use crate::vector::is_close;
use crate::vector::{Vec2, Vec3};
use crate::{
    components::Transform,
    constants::ZERO,
    draw::clipping::boundaries::{self, calc_boundaries},
    geometry::{
        shape::{
            buildshapes::{remove_face, ShapeBuilder},
            single_face::make_line_shape,
        },
        Plane,
    },
    vector::VectorTrait,
};

use super::utils::{color_number, print_grid};

#[test]
fn test_single_face_boundaries() {
    fn assert_on_boundaries<V: VectorTrait>(face_normal: V, boundaries: &[Plane<V>]) {
        let mut hits = 0;
        for boundary in boundaries {
            if !V::is_close(boundary.normal, face_normal) {
                assert!(is_close(boundary.threshold, 0.0));
            } else {
                hits += 1;
            }
            //needs more asserts
            println!("{}", boundary)
        }
        // there should be exactly one boundary with the same normal as the face
        assert_eq!(hits, 1);
    }

    let shape = make_line_shape();
    let boundaries = calc_boundaries(Vec2::zero(), &shape, &[true]);
    assert_on_boundaries(shape.faces[0].normal(), &boundaries);

    println!("3d, Triangle");
    let shape = make_3d_triangle();
    let boundaries = calc_boundaries(Vec3::zero(), &shape, &[true]);
    assert_on_boundaries(shape.faces[0].normal(), &boundaries);

    println!("3d, Square");
    let shape = make_3d_square();
    let boundaries = calc_boundaries(Vec3::zero(), &shape, &[true]);
    assert_on_boundaries(shape.faces[0].normal(), &boundaries);
}

#[test]
fn test_bounded_regions() {
    type V = Vec3;
    let mut shape = ShapeBuilder::build_cube(1.0).build();
    shape = remove_face(shape, 5);
    //shape.modify(&Transform::identity().with_rotation(0, 1, 2.2));
    let camera_pos = -V::one_hot(1) * 1.0 + V::one_hot(0) * 0.75 + V::one_hot(2) * 0.6;
    let face_visibility: Vec<bool> = shape
        .faces
        .iter()
        .map(|f| f.plane().point_signed_distance(camera_pos) > ZERO)
        .collect();
    let boundaries = calc_boundaries(camera_pos, &shape, &face_visibility);
    print_grid(2.0, 41, |x, y| {
        let pos = V::new(x, y, ZERO);
        if shape.verts.iter().any(|&p| (p - pos).norm() < 0.1) {
            ".".black()
        } else if shape.faces.iter().any(|f| (f.center() - pos).norm() < 0.1) {
            "^".red()
        } else if (camera_pos - pos).norm() < 0.1 {
            "*".bright_green()
        } else {
            color_number(
                boundaries
                    .iter()
                    .map(|b| b.point_signed_distance(pos) > ZERO)
                    .filter(|x| *x)
                    .count(),
            )
        }
    });
    println!("n boundaries: {}", boundaries.len());
    for b in boundaries {
        println!("{}", b);
    }
    for v in shape.verts {
        println!("{:?}", v);
    }
}
