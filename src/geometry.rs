pub mod buildfloor;
pub mod shape;

pub use shape::{Shape,Face};

use std::fmt;
use crate::vector::{VectorTrait,MatrixTrait,Field,VecIndex,is_close};

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
impl<V: VectorTrait> Plane<V> {
    fn from_normal_and_point(normal: V, point: V) -> Self {
        Plane{normal, threshold: normal.dot(point)}
    }
    //returns closest plane + distance
    pub fn point_normal_distance<'a, I: Iterator<Item=&'a Plane<V>>>(point : V, planes: I) -> Option<(&'a Plane<V>, Field)> {
        planes.fold(
            None,
            |acc: Option<(&Plane<V>,Field)>, plane| {
                let this_dist = plane.normal.dot(point) - plane.threshold;
                Some(acc.map_or_else(
                    || (plane, this_dist),
                    |(best_plane,cur_dist)| match this_dist > cur_dist {
                            true => (plane, this_dist), false => (best_plane,cur_dist)
                        }
                ))}
        )
    }
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


