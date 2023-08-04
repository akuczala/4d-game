use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::geometry::shape::{Face, FaceIndex, Shape};
use crate::geometry::{line_plane_intersect, Line, Plane};
use crate::tests::utils::print_grid;
use crate::vector::{Field, VectorTrait};

use std::fmt;

use super::subface::InteriorSubFace;

#[derive(Clone, Serialize, Deserialize)]
pub struct ConvexSubfaces(pub Vec<InteriorSubFace>);
impl ConvexSubfaces {
    //find indices of (d-1) faces that are joined by a (d-2) edge
    fn calc_subfaces<V: VectorTrait>(faces: &[Face<V>]) -> ConvexSubfaces {
        let mut subfaces: Vec<InteriorSubFace> = Vec::new();
        if V::DIM == 2 {
            for i in 0..faces.len() {
                for j in 0..i {
                    if count_common_verts(&faces[i], &faces[j]) >= 1 {
                        subfaces.push(InteriorSubFace { faceis: (i, j) })
                    }
                }
            }
            return ConvexSubfaces(subfaces);
        }
        let n_target = match V::DIM {
            3 => 1,
            4 => 2,
            _ => panic!("Invalid dimension for computing subfaces"),
        };
        for i in 0..faces.len() {
            for j in 0..i {
                if count_common_edges(&faces[i], &faces[j]) >= n_target {
                    subfaces.push(InteriorSubFace { faceis: (i, j) })
                }
            }
        }
        ConvexSubfaces(subfaces)
    }
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Convex {
    pub subfaces: ConvexSubfaces,
}
impl Convex {
    pub fn new<V: VectorTrait>(faces: &[Face<V>]) -> Self {
        Convex {
            subfaces: ConvexSubfaces::calc_subfaces(faces),
        }
    }
    pub fn point_within<V: VectorTrait>(point: V, distance: Field, faces: &[Face<V>]) -> bool {
        faces
            .iter()
            .map(Face::plane)
            .all(|p| p.point_signed_distance(point) < distance)
    }
    //returns points of intersection with shape
    pub fn line_intersect<V: VectorTrait>(
        shape: &Shape<V>,
        line: &Line<V>,
        visible_only: bool,
        face_visibility: &[bool],
    ) -> Vec<V> {
        //for (face, _) in shape
        shape
            .faces
            .iter()
            .zip(face_visibility.iter())
            .filter(|(_, &visible)| !visible_only || visible)
            .flat_map(|(face, _)| line_plane_intersect(line, face.plane()))
            .filter(|p| crate::vector::is_close(shape.point_signed_distance(*p), 0.))
            .collect_vec()
    }
}

fn count_common_verts<V: VectorTrait>(face1: &Face<V>, face2: &Face<V>) -> usize {
    let total_verts = face1.vertis.len() + face2.vertis.len();
    let unique_verts = face1
        .vertis
        .iter()
        .chain(face2.vertis.iter())
        .unique()
        .count();
    total_verts - unique_verts
}

//returns closest face and distance to it
pub fn closest_face_distance<V: VectorTrait>(
    faces: &[Face<V>],
    point: V,
) -> Option<(&Face<V>, Field)> {
    faces
        .iter()
        .map(|face| (face, face.plane().point_signed_distance(point)))
        .reduce(|(f1, a), (f2, b)| match a > b {
            true => (f1, a),
            false => (f2, b),
        })
}

fn count_common_edges<V: VectorTrait>(face1: &Face<V>, face2: &Face<V>) -> usize {
    let total_edges = face1.edgeis.len() + face2.edgeis.len();
    let unique_edges = face1
        .edgeis
        .iter()
        .chain(face2.edgeis.iter())
        .unique()
        .count();
    total_edges - unique_edges
}

#[test]
fn test_point_within() {
    use crate::vector::Vec3;
    let point = Vec3::new(1.2, 1.2, 1.2);
    let shape = crate::geometry::shape::buildshapes::build_prism_3d::<Vec3>(1.0, 1.0, 5);
    for v in shape
        .faces
        .iter()
        .map(Face::plane)
        .map(|plane| plane.point_signed_distance(point))
    {
        println!("{}", v);
    }
    assert!(!Convex::point_within(point, 0., &shape.faces))
}

//prints points at different distances from prism
// TODO: check that the boundary between 2 and 3 in the printout is supposed to be uneven. Was it always like that?
// TODO: seeing another potential difference: a small square of ones in the center
#[test]
fn test_point_within2() {
    use crate::vector::{linspace, Vec3};
    use colored::*;
    let shape = crate::geometry::shape::buildshapes::build_prism_3d::<Vec3>(1.0, 1.0, 4);
    print_grid(2.0, 40, |x, y| {
        let point = Vec3::new(x, y, 0.);
        // let newstr = match shape.point_within(Vec3::new(x,y,0.),0.) {
        //   true => "+", false => "_"
        // };
        let (i, dist) = shape.point_facei_distance(point);
        //println!("{}",dist);
        //let newstr = match dist {a if a > 1. => "#", a if a > 0. => "+", a if a <= 0. => "_", _ => "^"};
        let mut newstr = match i {
            1 => "1".blue(),
            2 => "2".yellow(),
            3 => "3".cyan(),
            4 => "4".green(),
            _ => "_".red(),
        };
        if dist > 1. {
            newstr = "+".to_string().white();
        }
        newstr
    })
    //assert!(false); //forces cargo test to print this
    //assert!(!shape.point_within(point,0.))
}
