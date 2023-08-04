use std::iter;

use crate::{
    components::{Convex, Shape, ShapeType, SingleFace},
    constants::ZERO,
    geometry::{
        shape::{
            single_face::{self, make_line_shape},
            subface::{BoundarySubFace, InteriorSubFace, SubFace},
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
        ShapeType::Convex(ref convex) => vec![calc_convex_boundaries(
            convex,
            camera_pos,
            &shape.faces,
            face_visibility,
        )],
        ShapeType::Generic(ref generic) => calc_generic_boundaries(
            camera_pos,
            shape,
            face_visibility,
            &generic.face_subfaces,
            &generic.subfaces,
        ),
        ShapeType::SingleFace(_) => {
            // below demonstrates that we can think of SingleFace as a special case of Generic
            // we do unnecessary copying here with .get_subfaces()
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
                    .map(|&subface_index| {
                        calc_generic_subface_boundary(
                            camera_pos,
                            &shape.verts,
                            &shape.faces,
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
) -> ConvexBoundarySet<V> {
    let n_boundaries = convex.subfaces.0.len() + faces.len();
    let mut boundaries: Vec<Plane<V>> = Vec::with_capacity(n_boundaries);
    let subface_boundaries =
        convex
            .subfaces
            .0
            .iter()
            .filter_map(|InteriorSubFace { faceis: (fi0, fi1) }| {
                (face_visibility[*fi0] != face_visibility[*fi1])
                    .then(|| calc_convex_boundary(faces[*fi0].plane(), faces[*fi1].plane(), origin))
            });
    let face_boundaries = faces
        .iter()
        .zip(face_visibility.iter())
        .filter_map(|(face, visible)| (*visible).then(|| calc_face_boundary(origin, face)));
    boundaries.extend(subface_boundaries.chain(face_boundaries));
    ConvexBoundarySet(boundaries)
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

fn calc_boundary_subface_boundary<V: VectorTrait>(
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
    if boundary_normal.dot(face_center - origin) > ZERO {
        boundary_normal = -boundary_normal;
    }
    Plane {
        normal: boundary_normal,
        threshold: boundary_normal.dot(origin),
    }
}

fn calc_generic_interior_boundary<V: VectorTrait>(
    camera_pos: V,
    faces: &[Face<V>],
    face_index: FaceIndex,
    interior_subface: &InteriorSubFace,
) -> Plane<V> {
    let InteriorSubFace { faceis: (fi0, fi1) } = interior_subface;
    let convex_boundary =
        calc_convex_boundary(faces[*fi0].plane(), faces[*fi1].plane(), camera_pos);
    // make sure the normal is pointing away from the face center
    if convex_boundary.point_signed_distance(faces[face_index].center()) < ZERO {
        convex_boundary
    } else {
        convex_boundary.flip_normal()
    }
}

fn calc_generic_subface_boundary<V: VectorTrait>(
    camera_pos: V,
    verts: &[V],
    faces: &[Face<V>],
    subface: &SubFace<V>,
    face_index: FaceIndex,
) -> Plane<V> {
    // it's assumed that we've already established that one of the adjoining faces is visile
    match subface {
        SubFace::Interior(isf) => {
            calc_generic_interior_boundary(camera_pos, faces, face_index, isf)
        }
        SubFace::Boundary(bsf) => calc_boundary_subface_boundary(
            &bsf.vertis,
            camera_pos,
            verts,
            faces[bsf.facei].center(),
        ),
    }
}
