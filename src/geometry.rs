pub mod buildshapes;
pub mod buildfloor;
pub mod shape;
pub mod face_shape;
pub mod face;
pub mod transform;

pub use face::Face;
pub use shape::{Shape,SubFace};
pub use face_shape::FaceShape;

use std::fmt;
use crate::vector::{VectorTrait,MatrixTrait,Field,VecIndex,is_close};
use crate::clipping;
use itertools::Itertools;
//use std::ops::Index;

#[derive(Clone)]
pub struct Line<V : VectorTrait>(pub V,pub V);
impl<V : VectorTrait> fmt::Display for Line<V> {
        // This trait requires `fmt` with this exact signature.
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "Line({},{})", self.0,self.1)
        }
}
impl<V : VectorTrait> Line<V> {
    pub fn map<F,U>(&self, f : F) -> Line<U>
    where U : VectorTrait,
    F : Fn(V) -> U
    {
        Line(f(self.0),f(self.1))
    }
}
#[derive(Clone)]
pub struct Plane<V : VectorTrait> {
    pub normal : V,
    pub threshold : Field
}
impl<V : VectorTrait> fmt::Display for Plane<V> {
        // This trait requires `fmt` with this exact signature.
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "n={},th={}", self.normal,self.threshold)
        }
}

pub fn point_plane_normal_axis<V : VectorTrait>(point : &V, plane : &Plane<V>) -> Field {
    return plane.threshold - point.dot(plane.normal)
}
pub fn line_plane_intersect<V>(line : &Line<V>, plane : &Plane<V>) -> Option<V>
where V : VectorTrait
{
    let p0 = line.0; let p1 = line.1;
    let n = plane.normal; let th = plane.threshold;
    let p0n = p0.dot(n); let p1n = p1.dot(n);
    //line is contained in plane
    if is_close(p0n,0.) && is_close(p1n,0.) {
        return None;
    }
    let t = (p0n - th)/(p0n - p1n);
    //plane does not intersect line segment
    if t < 0. || t > 1. {
        return None;
    }
    Some(V::linterp(p0,p1,t))
}
pub struct Sphere<V : VectorTrait>{pos : V, radius : Field}


//returns either none or pair of intersecting points
//note that tm and p are NOT bound between 0 and 1
pub fn sphere_line_intersect<V : VectorTrait>(line : Line<V>, r : Field) -> Option<Line<V>> {

        let v0 = line.0;
        let v1 = line.1;
        let dv = v1 - v0;
        let dv_norm = dv.norm();
        let dv = dv / dv_norm;

        //in our case, sphere center is the origin
        let v0_rel = v0;  // - sphere_center
        let v0r_dv = v0_rel.dot(dv);

        let discr = (v0r_dv)*(v0r_dv) - v0_rel.dot(v0_rel) + r * r;

        //print('discr',discr)
        //no intersection with line
        if discr < 0. {
            return None;
        }
                

        let sqrt_discr = discr.sqrt();
        let tm = -v0r_dv - sqrt_discr;
        let tp = -v0r_dv + sqrt_discr;

        //print('tm,tp',tm,tp)
        //no intersection with line segment
        if tm > dv_norm && tp > dv_norm {
            return None;
        }
        if tm < 0. && tp < 0. {
            return None;
        }
        let intersect_points = Line(v0 + dv*tm, v0 + dv*tp);
        
        Some(intersect_points)
}

pub type VertIndex = usize;
pub type EdgeIndex = usize;
pub type FaceIndex = usize;

#[derive(Clone)]
pub struct Edge(pub VertIndex,pub VertIndex);
impl fmt::Display for Edge {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "Edge({},{})", self.0, self.1)
        }
}

fn calc_bball(pos: V, verts: &Vec<V>) -> clipping::BBall<V> {
    let radius = verts.iter().map(|v| v.norm_sq()).fold(0./0., Field::max).sqrt();
    clipping::BBall{pos, radius}
}
use crate::colors::Color;
pub trait ShapeTrait<V : VectorTrait>: specs::Component {
    //type FaceIterator: Iterator<Item=Face<V>>;
    fn transform(&mut self);
    fn update(&mut self);
    fn rotate(&mut self, axis1: VecIndex, axis2: VecIndex, angle : Field);
    fn set_pos(self, pos : &V) -> Self;
    fn get_pos(& self) -> &V;
    //fn get_faces(&self) -> &Vec<Face<V>>;
    fn get_faces(&self) -> &[Face<V>];
    fn get_edges(&self) -> &Vec<Edge>;
    fn get_verts(&self) -> &Vec<V>;
    fn stretch(&self, scales : &V) -> Self;
    fn update_visibility(&mut self, camera_pos : V, transparent : bool);
    fn set_color(self, color : Color) -> Self;
    fn calc_bball(&self) -> clipping::BBall<V> {
        let radius = self.get_verts().iter().map(|v| v.norm_sq()).fold(0./0., Field::max).sqrt();
        clipping::BBall{pos: *self.get_pos(), radius}
    }

    fn calc_boundaries(&self, origin : V) -> Vec<Plane<V>>;
    fn point_normal_distance(&self, point : V) -> (V, Field);
}


