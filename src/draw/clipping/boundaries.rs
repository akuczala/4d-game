use std::iter;

use crate::{
    components::{Convex, Shape, ShapeType, SingleFace},
    constants::ZERO,
    geometry::{
        shape::{convex::ConvexSubFace, VertIndex},
        Face, Plane,
    },
    vector::VectorTrait,
};

pub fn calc_boundaries<V: VectorTrait>(
    camera_pos: V,
    shape: &Shape<V>,
    face_visibility: &[bool],
) -> Vec<Plane<V>> {
    match &shape.shape_type {
        ShapeType::Convex(convex) => calc_convex_boundaries(
            &convex.subfaces.0,
            camera_pos,
            &shape.faces,
            face_visibility,
        ),
        ShapeType::SingleFace(single_face) => calc_single_face_boundaries(
            single_face,
            camera_pos,
            &shape.verts,
            shape.faces[0].center(),
            shape.faces[0].normal(),
            face_visibility[0],
        ),
    }
}

fn calc_convex_boundary<V: VectorTrait>(face1: &Plane<V>, face2: &Plane<V>, origin: V) -> Plane<V> {
    let (n1, n2) = (face1.normal, face2.normal);
    let (th1, th2) = (face1.threshold, face2.threshold);

    //k1 and k2 must have opposite signs
    let k1 = n1.dot(origin) - th1;
    let k2 = n2.dot(origin) - th2;
    //assert!(k1*k2 < 0.0,"k1 = {}, k2 = {}",k1,k2);

    let t = k1 / (k1 - k2);

    let n3 = V::linterp(n1, n2, t);
    let th3 = crate::vector::scalar_linterp(th1, th2, t);

    Plane {
        normal: n3,
        threshold: th3,
    }
}

fn calc_convex_boundaries<V: VectorTrait>(
    subfaces: &[ConvexSubFace],
    origin: V,
    faces: &[Face<V>],
    face_visibility: &[bool],
) -> Vec<Plane<V>> {
    let mut boundaries: Vec<Plane<V>> = Vec::new();

    for subface in subfaces {
        let face1 = &faces[subface.faceis.0];
        let face2 = &faces[subface.faceis.1];
        if face_visibility[subface.faceis.0] != face_visibility[subface.faceis.1] {
            let boundary = calc_convex_boundary(face1.plane(), face2.plane(), origin);
            boundaries.push(boundary);
        }
    }
    //visible faces are boundaries
    for (face, visible) in faces.iter().zip(face_visibility.iter()) {
        if *visible {
            boundaries.push(face.plane().clone())
        }
    }
    boundaries
}

fn calc_single_face_boundary<V: VectorTrait>(
    subface_vertis: &[VertIndex],
    origin: V,
    verts: &[V],
    face_center: V,
) -> Plane<V> {
    let mut boundary_normal = V::cross_product(
        subface_vertis
            .iter()
            .take((V::DIM.unsigned_abs() - 1) as usize)
            .map(|&vi| verts[vi] - origin),
    )
    .normalize();
    //not sure about the sign here
    if boundary_normal.dot(face_center - origin) > 0. {
        boundary_normal = -boundary_normal;
    }
    Plane {
        normal: boundary_normal,
        threshold: boundary_normal.dot(origin),
    }
}
fn calc_single_face_boundaries<V: VectorTrait>(
    single_face: &SingleFace<V>,
    origin: V,
    verts: &[V],
    face_center: V,
    face_normal: V,
    visible: bool,
) -> Vec<Plane<V>> {
    if visible {
        single_face
            .subfaces
            .0
            .iter()
            .map(|subface| calc_single_face_boundary(&subface.vertis, origin, verts, face_center))
            // face is itself a boundary
            .chain(iter::once(Plane::from_normal_and_point(
                if single_face.two_sided && (face_normal.dot(origin - face_center) < ZERO) {
                    -face_normal
                } else {
                    face_normal
                },
                face_center,
            )))
            .collect()
    } else {
        Vec::new()
    }
}
// TODO: fix these tests
#[test]
fn test_single_face_boundaries() {
    use crate::geometry::shape::single_face::{make_3d_square, make_3d_triangle};
    use crate::geometry::shape::Edge;
    use crate::vector::is_close;
    use crate::vector::{Vec2, Vec3};

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

    // Line segment
    let shape = Shape::new_convex(
        vec![Vec2::new(1., -1.), Vec2::new(1., 1.)],
        vec![Edge(0, 1)],
        vec![Face::new(vec![0], Vec2::new(-1., 0.))],
    );
    let subfaces_vertis = vec![vec![0], vec![1]];
    let single_face = SingleFace::new(
        &shape.verts,
        shape.faces[0].normal(),
        &subfaces_vertis,
        false,
    );
    let boundaries = calc_single_face_boundaries(
        &single_face,
        Vec2::zero(),
        &shape.verts,
        shape.faces[0].center(),
        shape.faces[0].normal(),
        true,
    );
    assert_on_boundaries(shape.faces[0].normal(), &boundaries);

    println!("3d, Triangle");
    let (shape, single_face) = make_3d_triangle();
    let boundaries = calc_single_face_boundaries(
        &single_face,
        Vec3::zero(),
        &shape.verts,
        shape.faces[0].center(),
        shape.faces[0].normal(),
        true,
    );
    assert_on_boundaries(shape.faces[0].normal(), &boundaries);

    println!("3d, Square");
    let (shape, single_face) = make_3d_square();
    let boundaries = calc_single_face_boundaries(
        &single_face,
        Vec3::zero(),
        &shape.verts,
        shape.faces[0].center(),
        shape.faces[0].normal(),
        true,
    );
    assert_on_boundaries(shape.faces[0].normal(), &boundaries);
}
