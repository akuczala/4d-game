mod affine_transform;
pub mod buildfloor;
pub mod shape;
pub mod transform;
use crate::vector::{barycenter, is_close, scalar_linterp, Field, VectorTrait};
use serde::{Deserialize, Serialize};
pub use shape::{Face, Shape};
use std::fmt;
pub use transform::{Transform, Transformable};

#[derive(Clone)]
pub struct Line<V>(pub V, pub V);
impl<V: fmt::Display> fmt::Display for Line<V> {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Line({},{})", self.0, self.1)
    }
}
impl<V: Copy> Line<V> {
    pub fn map<F, U>(&self, f: F) -> Line<U>
    where
        F: Fn(V) -> U,
    {
        Line(f(self.0), f(self.1))
    }
    pub fn zip_map<F, U: Copy, W>(&self, other: Line<U>, f: F) -> Line<W>
    where
        F: Fn(V, U) -> W,
    {
        Line(f(self.0, other.0), f(self.1, other.1))
    }
}
impl<V: VectorTrait> Line<V> {
    pub fn is_close(&self, other: &Line<V>) -> bool {
        V::is_close(self.0, other.0) && V::is_close(self.1, other.1)
    }
    pub fn linterp(&self, t: Field) -> V {
        V::linterp(self.0, self.1, t)
    }
}
impl Line<Field> {
    pub fn linterp(&self, t: Field) -> Field {
        scalar_linterp(self.0, self.1, t)
    }
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Plane<V> {
    pub normal: V,
    pub threshold: Field,
}
impl<V: VectorTrait> Plane<V> {
    fn from_normal_and_point(normal: V, point: V) -> Self {
        Plane {
            normal,
            threshold: normal.dot(point),
        }
    }
    fn from_points_and_vec(points: &Vec<V>, normal_dir: V) -> Self {
        // take D points, then subtract one of these from the others to get
        // D-1 vectors parallel to the plane

        //todo this won't work if the points all lie in the same d-2 plane
        let v0: V = points[0];
        let d = V::DIM.unsigned_abs() as usize;
        let parallel_vecs = points[1..d].iter().map(|&v| v - v0);
        let mut normal = V::cross_product(parallel_vecs).normalize();
        if normal.dot(normal_dir) < 0.0 {
            //normal should parallel to normal_dir
            normal = -normal;
        }
        let center = barycenter(points);
        Plane::from_normal_and_point(normal, center)
    }
    pub fn point_signed_distance(&self, point: V) -> Field {
        self.normal.dot(point) - self.threshold
    }
    //returns closest plane + distance
    pub fn point_normal_distance<'a, I: Iterator<Item = &'a Plane<V>>>(
        point: V,
        planes: I,
    ) -> Option<(&'a Plane<V>, Field)> {
        planes.fold(None, |acc: Option<(&Plane<V>, Field)>, plane| {
            let this_dist = plane.point_signed_distance(point);
            Some(acc.map_or_else(
                || (plane, this_dist),
                |(best_plane, cur_dist)| match this_dist > cur_dist {
                    true => (plane, this_dist),
                    false => (best_plane, cur_dist),
                },
            ))
        })
    }
}
impl<V: fmt::Display> fmt::Display for Plane<V> {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "n={},th={}", self.normal, self.threshold)
    }
}

fn is_point_in_sphere<V: VectorTrait>(r: Field, p: V) -> bool {
    p.norm_sq() < r * r
}

pub fn point_plane_normal_axis<V: VectorTrait>(point: &V, plane: &Plane<V>) -> Field {
    plane.threshold - point.dot(plane.normal)
}
pub fn line_plane_intersect<V>(line: &Line<V>, plane: &Plane<V>) -> Option<V>
where
    V: VectorTrait,
{
    let p0 = line.0;
    let p1 = line.1;
    let n = plane.normal;
    let th = plane.threshold;
    let p0n = p0.dot(n);
    let p1n = p1.dot(n);
    //line is contained in plane
    if is_close(p0n, 0.) && is_close(p1n, 0.) {
        return None;
    }
    let t = (p0n - th) / (p0n - p1n);
    //plane does not intersect line segment
    if !(0. ..=1.).contains(&t) {
        return None;
    }
    Some(V::linterp(p0, p1, t))
}
pub struct Sphere<V: VectorTrait> {
    pos: V,
    radius: Field,
}

//returns either none or pair of intersecting points

pub fn sphere_line_intersect<V: VectorTrait>(line: Line<V>, r: Field) -> Option<Line<V>> {
    fn t_in_range(t: Field) -> bool {
        0.0 < t && t < 1.0
    }
    let v0 = line.0;
    let dv = line.1 - line.0;
    // shouldn't this return None if the line segment is within the sphere?
    // not what we want, but is how im understanding this
    sphere_t_intersect_infinite_normed(line, r)
        .filter(|Line(tm, tp)| !((*tm < 0.0 && *tp < 0.0) || (*tm > 1.0 && *tp > 1.0)))
        .map(|Line(tm, tp)| Line(v0 + dv * tm, v0 + dv * tp))
}

// returns either none or (pair of roots, normed dv)
//note that tm and tp are NOT bound between 0 and 1
// tm and tp are in units of distance; the line segment goes from t=0 to t=dv.norm()
// this fn returns None if the segment is completely outside the sphere, and returns both tm and tp if it is
// inside or partially inside
// not used
pub fn sphere_t_intersect<V: VectorTrait>(line: Line<V>, r: Field) -> Option<Line<Field>> {
    let v0 = line.0;
    let v1 = line.1;
    let dv = v1 - v0;
    let dv_norm = dv.norm();
    let dv = dv / dv_norm;

    let v0r_dv = v0.dot(dv);

    let discr = (v0r_dv) * (v0r_dv) - v0.dot(v0) + r * r;

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
    Some(Line(tm, tp))
}

// normalize t from 0 to 1 (line.0 to line.1)
// returns None only if inifinte extended line does not intersect sphere
// we really only need v0 and (v1 - v0) as arguments
// returns Line(0, 1) if v0 == v1
pub fn sphere_t_intersect_infinite_normed<V: VectorTrait>(
    line: Line<V>,
    r: Field,
) -> Option<Line<Field>> {
    let v0 = line.0;
    let v1 = line.1;
    let dv = v1 - v0;
    let dv_norm = dv.norm();

    // handle degenerate case
    if is_close(dv_norm, 0.0) {
        return if is_point_in_sphere(r, v0) {
            Some(Line(0.0, 1.0)) // not really any good value to put here
        } else {
            None
        };
    }
    let dv = dv / dv_norm;

    let v0r_dv = v0.dot(dv);

    let discr = (v0r_dv) * (v0r_dv) - v0.dot(v0) + r * r;

    //no intersection with line
    if discr < 0. {
        return None;
    }

    let sqrt_discr = discr.sqrt();
    let tm = (-v0r_dv - sqrt_discr) / dv_norm;
    let tp = (-v0r_dv + sqrt_discr) / dv_norm;

    Some(Line(tm, tp))
}

#[test]
fn test_sphere_t_infinite_normed() {
    use crate::vector::Vec2;
    let line = Line(Vec2::new(-0.3333, -1.0), Vec2::new(-0.3333, 1.666));
    let roots = sphere_t_intersect_infinite_normed(line, 0.5);
    println!("{}", roots.unwrap())
}

#[test]
fn test_sphere_line_intersect() {
    use crate::vector::Vec2;
    let line = Line(Vec2::new(-0.2, -0.2), Vec2::new(0.2, 0.2));
    let v = sphere_line_intersect(line, 0.5);
    println!("{}", v.unwrap())
}
