use std::iter;

use crate::{
    components::{Convex, Shape, ShapeType, SingleFace},
    constants::ZERO,
    geometry::{
        shape::{
            convex::ConvexSubFace,
            single_face::{self, make_line_shape, BoundarySubFace},
            subface::SubFace,
            FaceIndex, VertIndex,
        },
        Face, Plane,
    },
    vector::VectorTrait,
};

/// Represents a set of boundaries forming an unbounded convex region
/// These are cast by convex volumes or faces
pub struct ConvexBoundarySet<V>(pub Vec<Plane<V>>);

// need to group boundaries into convex regions
// the easiest way to do that is to group by the casting (convex) face or by convex volume
pub fn calc_boundaries<V: VectorTrait>(
    camera_pos: V,
    shape: &Shape<V>,
    face_visibility: &[bool],
) -> Vec<ConvexBoundarySet<V>> {
    match shape.shape_type {
        ShapeType::Convex(ref convex) => vec![ConvexBoundarySet(calc_convex_boundaries(
            convex,
            camera_pos,
            &shape.faces,
            face_visibility,
        ))],
        ShapeType::Generic(ref generic) => calc_generic_boundaries(
            camera_pos,
            shape,
            face_visibility,
            &generic.face_subfaces,
            &generic.subfaces,
        ),
        ShapeType::SingleFace(_) => {
            let subfaces = shape.shape_type.get_subfaces();
            calc_generic_boundaries(
                camera_pos,
                shape,
                face_visibility,
                &[(0..subfaces.len()).collect()],
                &subfaces,
            )
        }
    }
}

fn calc_subface_boundary<V: VectorTrait>(
    camera_pos: V,
    verts: &[V],
    faces: &[Face<V>],
    face_visibility: &[bool],
    subface: &SubFace<V>,
    face_index: FaceIndex,
) -> Option<Plane<V>> {
    match subface {
        SubFace::Interior(ConvexSubFace { faceis }) => {
            let convex_boundary =
                calc_convex_boundary(faces[faceis.0].plane(), faces[faceis.1].plane(), camera_pos);
            // make sure the normal is pointing away from the face center
            if convex_boundary.point_signed_distance(faces[face_index].center()) < ZERO {
                Some(convex_boundary)
            } else {
                Some(convex_boundary.flip_normal())
            }
        }
        SubFace::Boundary(bsf) => (face_visibility[bsf.facei]).then(|| {
            calc_single_face_boundary(&bsf.vertis, camera_pos, verts, faces[bsf.facei].center())
        }),
    }
}
fn calc_face_boundary<V: VectorTrait>(camera_pos: V, face: &Face<V>) -> Plane<V> {
    Plane::from_normal_and_point(
        if face.two_sided && (face.normal().dot(camera_pos - face.center()) < ZERO) {
            -face.normal()
        } else {
            face.normal()
        },
        face.center(),
    )
}

fn calc_generic_boundaries<V: VectorTrait>(
    camera_pos: V,
    shape: &Shape<V>,
    face_visibility: &[bool],
    face_subfaces: &[Vec<usize>],
    subfaces: &[SubFace<V>],
) -> Vec<ConvexBoundarySet<V>> {
    izip!(shape.faces.iter(), face_visibility, face_subfaces)
        .enumerate()
        .filter(|(_, (_, &visible, _))| visible)
        .map(|(face_index, (face, _, subface_indexes))| {
            ConvexBoundarySet(
                subface_indexes
                    .iter()
                    .flat_map(|&subface_index| {
                        calc_subface_boundary(
                            camera_pos,
                            &shape.verts,
                            &shape.faces,
                            face_visibility,
                            &subfaces[subface_index],
                            face_index,
                        )
                    })
                    .chain(iter::once(calc_face_boundary(camera_pos, face)))
                    .collect(),
            )
        })
        .collect()
}

fn calc_convex_boundaries<V: VectorTrait>(
    convex: &Convex,
    origin: V,
    faces: &[Face<V>],
    face_visibility: &[bool],
) -> Vec<Plane<V>> {
    let mut boundaries: Vec<Plane<V>> = Vec::new();

    for subface in &convex.subfaces.0 {
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
