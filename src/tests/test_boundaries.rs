use colored::Colorize;
use itertools::Itertools;

use crate::components::ShapeType;
use crate::draw::clipping::boundaries::ConvexBoundarySet;
use crate::draw::clipping::{clip_line, clip_line_convex};
use crate::geometry::shape::buildshapes::invert_normals;
use crate::geometry::shape::single_face::{make_3d_square, make_3d_triangle};
use crate::geometry::shape::Edge;
use crate::geometry::Line;
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
    fn assert_on_boundaries<V: VectorTrait>(face_normal: V, boundaries: &[ConvexBoundarySet<V>]) {
        assert_eq!(boundaries.len(), 1);
        let boundaries = &boundaries[0].0;
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
    type V = Vec2;
    let mut shape = ShapeBuilder::build_cube(1.0).build();
    shape = invert_normals(&shape);
    shape = remove_face(shape, 3);
    //shape = invert_normals(&shape);
    //let shape = make_line_shape().to_generic();
    if let ShapeType::Generic(gst) = &shape.shape_type {
        println!("{}", serde_json::to_string(&gst).unwrap());
    }
    //shape.modify(&Transform::identity().with_rotation(0, 1, 2.2));
    let camera_pos = -V::one_hot(1) * 2.1; //+ V::one_hot(2) * 0.6;
    let face_visibility: Vec<bool> = shape
        .faces
        .iter()
        .map(|f| f.plane().point_signed_distance(camera_pos) > ZERO)
        .collect();
    let boundaries = calc_boundaries(camera_pos, &shape, &face_visibility);
    assert!(boundaries.len() > 0);
    print_grid(2.0, 41, |x, y| {
        let pos = V::new(x, y);
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
                    .filter(|cbs| cbs.0.iter().all(|b| b.point_signed_distance(pos) <= ZERO))
                    .count(),
            )
            // color_number(
            //     boundaries[1]
            //         .0
            //         .iter()
            //         .map(|b| b.point_signed_distance(pos) > ZERO)
            //         .filter(|x| *x)
            //         .count(),
            // )
        }
    });
    println!("n boundaries: {}", boundaries.len());
    for (i, cbs) in boundaries.iter().enumerate() {
        println!("Set {}", i);
        for b in &cbs.0 {
            println!("{}", b);
        }
    }
    // clip line
    let line = Line(V::new(-0.7, 1.4), V::new(0.7, 1.4));
    // println!("{}", serde_json::to_string(&[line.clone()]).unwrap());

    let clipped_lines = clip_line(line, &boundaries);

    //let clipped_lines = clip_line_convex(line, &boundaries[1].0).into_iter().collect_vec();
    //println!("{}", serde_json::to_string(&clipped_lines).unwrap());

    // let clipped_lines = clip_line_convex(line, &boundaries[0].0).into_iter().collect_vec();
    // println!("{}", serde_json::to_string(&clipped_lines).unwrap());
    // let clipped_lines = clipped_lines
    //     .into_iter()
    //     .flat_map(
    //         |line| clip_line_convex(line, &boundaries[1].0)
    //     )
    //     .collect_vec();
    // println!("{}", serde_json::to_string(&clipped_lines).unwrap());
    // let clipped_lines = clipped_lines
    //     .into_iter()
    //     .flat_map(
    //         |line| clip_line_convex(line, &boundaries[2].0)
    //     )
    //     .collect_vec();

    println!("{}", serde_json::to_string(&clipped_lines).unwrap());

    // let line = Line(V::new(-0.7, 1.0), V::new(0.6, 1.0));
    // let clipped_lines = clip_line_convex(line, &boundaries[1].0).into_iter().collect_vec();
    // println!("{}", serde_json::to_string(&clipped_lines).unwrap());
}
