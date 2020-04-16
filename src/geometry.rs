pub mod buildshapes;
pub mod buildfloor;
pub mod face;
pub mod mesh;
use std::fmt;
use crate::vector::{VectorTrait,MatrixTrait,Field,is_close};
use std::clone::Clone;
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

pub fn line_plane_intersect<V>(line : Line<V>, plane : Plane<V>) -> Option<V>
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
