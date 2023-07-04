use itertools::Itertools;

use crate::geometry::shape::{Face, FaceIndex, Shape};
use crate::geometry::{line_plane_intersect, Line, Plane};
use crate::vector::{Field, VectorTrait};

#[derive(Clone)]
struct SubFace {
    pub faceis: (FaceIndex, FaceIndex),
}
use std::fmt;
impl fmt::Display for SubFace {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SubFace({},{})", self.faceis.0, self.faceis.1)
    }
}

#[derive(Clone)]
struct Subfaces(pub Vec<SubFace>);
impl Subfaces {
    //find indices of (d-1) faces that are joined by a (d-2) edge
    fn calc_subfaces<V: VectorTrait>(faces: &Vec<Face<V>>) -> Subfaces {
        let mut subfaces: Vec<SubFace> = Vec::new();
        if V::DIM == 2 {
            for i in 0..faces.len() {
                for j in 0..i {
                    if count_common_verts(&faces[i], &faces[j]) >= 1 {
                        subfaces.push(SubFace { faceis: (i, j) })
                    }
                }
            }
            return Subfaces(subfaces);
        }
        let n_target = match V::DIM {
            3 => 1,
            4 => 2,
            _ => panic!("Invalid dimension for computing subfaces"),
        };
        for i in 0..faces.len() {
            for j in 0..i {
                if count_common_edges(&faces[i], &faces[j]) >= n_target {
                    subfaces.push(SubFace { faceis: (i, j) })
                }
            }
        }
        Subfaces(subfaces)
    }
}
#[derive(Clone)]
pub struct Convex {
    subfaces: Subfaces,
}
impl Convex {
    pub fn new<V: VectorTrait>(shape: &Shape<V>) -> Self {
        Convex {
            subfaces: Subfaces::calc_subfaces(&shape.faces),
        }
    }
    fn calc_boundary<V: VectorTrait>(face1: &Plane<V>, face2: &Plane<V>, origin: V) -> Plane<V> {
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
    pub fn calc_boundaries<V: VectorTrait>(
        &self,
        origin: V,
        faces: &Vec<Face<V>>,
        face_visibility: &Vec<bool>,
    ) -> Vec<Plane<V>> {
        let mut boundaries: Vec<Plane<V>> = Vec::new();

        for subface in &self.subfaces.0 {
            let face1 = &faces[subface.faceis.0];
            let face2 = &faces[subface.faceis.1];
            if face_visibility[subface.faceis.0] == !face_visibility[subface.faceis.1] {
                let boundary = Self::calc_boundary(face1.plane(), face2.plane(), origin);
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
    pub fn point_within<V: VectorTrait>(point: V, distance: Field, faces: &Vec<Face<V>>) -> bool {
        faces
            .iter()
            .map(Face::plane)
            .all(|p| p.point_signed_distance(point) < distance)
    }
    //returns points of intersection with shape
    pub fn line_intersect<V: VectorTrait>(
        &self,
        shape: &Shape<V>,
        line: &Line<V>,
        visible_only: bool,
        face_visibility: &[bool],
    ) -> Vec<V> {
        //impl std::iter::Iterator<Item=Option<V>> {
        let mut out_points = Vec::<V>::new();
        for (face, _) in shape
            .faces
            .iter()
            .zip(face_visibility.iter())
            .filter(|(_, &visible)| !visible_only || visible)
        {
            if let Some(p) = line_plane_intersect(line, face.plane()) {
                if crate::vector::is_close(shape.point_signed_distance(p), 0.) {
                    out_points.push(p);
                }
            }
        }
        out_points
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
#[test]
fn test_point_within2() {
    use crate::vector::{linspace, Vec3};
    use colored::*;
    let shape = crate::geometry::shape::buildshapes::build_prism_3d::<Vec3>(1.0, 1.0, 4);
    for x in linspace(-2., 2., 40) {
        let mut line = "".to_string();
        for y in linspace(-2., 2., 40) {
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
            line = format!("{} {}", line, newstr);
        }
        println!("{}", line);
    }
    //assert!(false); //forces cargo test to print this
    //assert!(!shape.point_within(point,0.))
}
