pub mod bball;
pub mod boundaries;

use crate::ecs_utils::Componentable;

use crate::graphics::colors::WHITE;
use crate::vector::{Field, VecIndex, VectorTrait};
use std::collections::{HashMap, HashSet};

use crate::components::Shape;
use crate::draw::DrawLine;
use crate::geometry::{sphere_t_intersect_infinite_normed, Line, Plane};

use specs::{Entities, Entity, Join, ReadStorage, WriteStorage};

use self::bball::BBall;
use self::boundaries::ConvexBoundarySet;

use super::Scratch;

// TODO: split into smaller modules
pub struct ClipState<V> {
    //pub in_front : Vec<Vec<bool>>,
    //pub separators : Vec<Vec<Separator<V>>>,
    //pub in_front : HashSet<(Entity,Entity)>, //needs to be cleared whenever # shapes changes
    //pub separators : HashMap<(Entity,Entity),Separator<V>>, //ditto
    //pub separations_debug : Vec<Vec<Separation>>, //don't need this, but is useful for debug
    pub clipping_enabled: bool,
    phantom: std::marker::PhantomData<V>,
}
//could alternatively hold in a hash map over (entity,entity) pairs

impl<V> Default for ClipState<V> {
    fn default() -> Self {
        ClipState::new()
    }
}

impl<V> ClipState<V> {
    pub fn new() -> Self {
        //let shapes : Vec<&Shape<V>> = (&read_shapes).join().collect();
        ClipState {
            //in_front : HashSet::new(),
            //separations_debug :vec![vec![Separation::Unknown ; shapes_len] ; shapes_len],
            //separators : HashMap::new(),
            clipping_enabled: true,
            phantom: std::marker::PhantomData::<V>,
        }
    }
    // #[allow(dead_code)]
    // pub fn print_debug(&self) {
    //     for row in self.separations_debug.iter() {
    //         println!("");
    //         for val in row.iter() {
    //                 print!("{}, ",match val {
    //                     Separation::S1Front => "1",
    //                     Separation::S2Front => "2",
    //                     Separation::NoFront => "_",
    //                     Separation::Unknown => "U"
    //                 });
    //             }
    //         }
    //         println!("");
    // }
}
pub struct ShapeClipState<V> {
    pub in_front: HashSet<Entity>,
    pub separators: HashMap<Entity, Separator<V>>,
    pub boundaries: Vec<ConvexBoundarySet<V>>,
    pub transparent: bool,
    pub face_visibility: Vec<bool>,
}

impl<V: VectorTrait> Default for ShapeClipState<V> {
    fn default() -> Self {
        Self {
            in_front: HashSet::new(),
            separators: HashMap::new(),
            boundaries: Vec::new(),
            transparent: false,
            face_visibility: Vec::new(),
        }
    }
}

impl<V: VectorTrait> ShapeClipState<V> {
    pub fn remove(&mut self, e: &Entity) {
        self.in_front.remove(e);
        self.separators.remove(e);
    }
}
#[derive(Clone, Copy)]
pub struct InFrontArg<'a, V> {
    pub shape: &'a Shape<V>,
    pub bball: &'a BBall<V>,
    pub entity: Entity,
}

//i've avoiding double mutable borrowing here by passing the entire shape_clip_states to calc_in_front_pair
//a disadvantage here is that we have no guarantee that the processed entities have the ShapeClipState component
//and that we have to iterate over all entities with the Shape component, instead of just those with both Shape and ShapeClipState
//but for now, every shape has a ShapeClipState.
pub fn calc_in_front<V: VectorTrait + Componentable>(
    read_shapes: &ReadStorage<Shape<V>>,
    read_bballs: &ReadStorage<BBall<V>>,
    shape_clip_states: &mut WriteStorage<ShapeClipState<V>>,
    entities: &Entities,
    origin: &V,
) {
    //collect a vec of references to shapes
    //let shapes : Vec<&Shape<V>> = (& read_shapes).join().collect();
    //loop over unique pairs
    for (shape1, bball1, e1) in (read_shapes, read_bballs, entities).join() {
        for (shape2, bball2, e2) in (read_shapes, read_bballs, entities)
            .join()
            .filter(|(_sh, _bb, e)| *e > e1)
        {
            calc_in_front_pair(
                InFrontArg {
                    shape: shape1,
                    bball: bball1,
                    entity: e1,
                },
                InFrontArg {
                    shape: shape2,
                    bball: bball2,
                    entity: e2,
                },
                shape_clip_states,
                origin,
            )
        }
    }
}

pub fn calc_in_front_pair<'a, V: VectorTrait + Componentable>(
    a: InFrontArg<'a, V>,
    b: InFrontArg<'a, V>,
    shape_clip_states: &mut WriteStorage<ShapeClipState<V>>,
    origin: &V,
) {
    //try dynamic separation
    let mut sep_state = dynamic_separate(a.bball, b.bball, origin);
    let is_unknown = matches!(sep_state, Separation::Unknown);
    //if that's unsuccessful, try static separation
    if is_unknown {
        let a_clip_state = shape_clip_states.get_mut(a.entity).unwrap();
        //let mut a_separators = shape_clip_states.get_mut(a.entity).unwrap().separators;
        let maybe_sep = a_clip_state.separators.get_mut(&b.entity);

        //compute static separator if it hasn't been computed yet
        let sep = match maybe_sep {
            Some(s) => *s,
            None => {
                let s = separate_between_centers(a, b);
                a_clip_state.separators.insert(b.entity, s);
                s
            }
        };

        //determine separation state from separator
        sep_state = sep.apply(origin);
    };
    let new_vals = match sep_state {
        Separation::S1Front => (true, false),
        Separation::S2Front => (false, true),
        Separation::NoFront => (false, false),
        Separation::Unknown => (true, true),
    };
    {
        let a_clip_state = shape_clip_states.get_mut(a.entity).unwrap();
        match new_vals.0 {
            true => a_clip_state.in_front.insert(b.entity),
            false => a_clip_state.in_front.remove(&b.entity),
        };
    }
    {
        let b_clip_state = shape_clip_states.get_mut(b.entity).unwrap();
        match new_vals.1 {
            true => b_clip_state.in_front.insert(a.entity),
            false => b_clip_state.in_front.remove(&a.entity),
        };
    }
}

pub fn clip_line_plane<V>(line: Line<V>, plane: &Plane<V>, small_z: Field) -> Option<Line<V>>
where
    V: VectorTrait,
{
    let Line(p0, p1) = line;

    let n = plane.normal;
    let th = plane.threshold + small_z;

    let (p0n, p1n) = (p0.dot(n), p1.dot(n));
    let (p0_safe, p1_safe) = (p0n >= th, p1n >= th);
    //both points behind
    if !p0_safe && !p1_safe {
        return None;
    }
    //both points in front
    if p0_safe && p1_safe {
        return Some(line);
    }
    //otherwise only one of the vertices is behind the camera
    let t_intersect = (p0n - th) / (p0n - p1n);
    let intersect = V::linterp(p0, p1, t_intersect);
    if (!p0_safe) && p1_safe {
        Some(Line(intersect, p1))
    } else {
        Some(Line(p0, intersect))
    }
}

pub fn clip_line_cube<V: VectorTrait>(line: Line<V>, r: Field) -> Option<Line<V>> {
    //construct the d cube planes, normals facing in
    let planes_iter = (0..V::DIM).flat_map(move |i| {
        ([-1., 1.]).iter().map(move |&sign| Plane {
            normal: V::one_hot(i) * sign,
            threshold: -r,
        })
    });
    //successively clip on each plane
    let mut clipped_line = Some(line);
    for plane in planes_iter {
        clipped_line = clipped_line.and_then(|line| clip_line_plane(line, &plane, 0.));
    }
    clipped_line
    // todo: fold
    // planes_iter.fold(
    //     Some(line),
    //     |clipped, plane| clipped.and_then(
    //         |line| clip_line_plane(line, &plane, 0.0)
    //     )
    // )
}

pub fn clip_line_sphere<V: VectorTrait>(line: Line<V>, r: Field) -> Option<Line<V>> {
    let v0 = line.0;
    let v1 = line.1;

    let v0_in_sphere = v0.dot(v0) < r * r;
    let v1_in_sphere = v1.dot(v1) < r * r;

    if v0_in_sphere && v1_in_sphere {
        return Some(line);
    }

    let intersect = crate::geometry::sphere_line_intersect(line, r);
    intersect.map(|iline: Line<V>| match (v0_in_sphere, v1_in_sphere) {
        (false, false) => iline,
        (false, true) => Line(iline.0, v1),
        (true, false) => Line(v0, iline.1),
        (true, true) => iline, // will never reach this case (handled above)
    })
}
pub fn clip_line_cylinder<V: VectorTrait>(line: Line<V>, r: Field, h: Field) -> Option<Line<V>> {
    //first clip with planes on top and bottom
    let long_axis = 1;
    let planes_iter = ([-1., 1.]).iter().map(move |&sign| Plane {
        normal: V::one_hot(long_axis) * sign,
        threshold: -h,
    });
    let mut clipped_line = Some(line);
    for plane in planes_iter {
        clipped_line = clipped_line.and_then(|line| clip_line_plane(line, &plane, 0.));
    }
    clipped_line.and_then(|l: Line<V>| clip_line_tube(l, r))
}

// clip line in infinite cylinder
// TODO: reduce # vec allocations
pub fn clip_line_tube<V: VectorTrait>(line: Line<V>, r: Field) -> Option<Line<V>> {
    fn build_vec<V: VectorTrait>(u: V::SubV, a: Field, long_axis: VecIndex) -> V {
        let mut u_iter = u.iter();
        V::from_iter(
            (0..V::DIM)
                .map(|i| {
                    if i == long_axis {
                        a
                    } else {
                        *u_iter.next().unwrap()
                    }
                })
                .collect::<Vec<Field>>()
                .iter(),
        )
    }
    let long_axis = 1;
    // this kind of shit, where we're just dropping an index, should be a library fn
    // this is also probably not very fast
    let proj_line: Line<V::SubV> = line.map(|p| {
        V::SubV::from_iter(
            (0..V::DIM)
                .filter(|&i| i != long_axis)
                .map(|i| p[i])
                .collect::<Vec<Field>>()
                .iter(),
        )
    });
    let perp = line.map(|p| p[long_axis]);
    let t_roots = sphere_t_intersect_infinite_normed(proj_line.clone(), r);
    fn t_in_range(t: Field) -> bool {
        0.0 < t && t < 1.0
    }
    t_roots
        .filter(
            // eliminate lines segments outside the circle entirely
            |Line(tm, tp)| !((*tm < 0.0 && *tp < 0.0) || (*tm > 1.0 && *tp > 1.0)),
        )
        .map(|Line(tm, tp)| match (t_in_range(tm), t_in_range(tp)) {
            // line segment passes all the way through the circle
            (true, true) => Line(
                build_vec(proj_line.linterp(tm), perp.linterp(tm), long_axis),
                build_vec(proj_line.linterp(tp), perp.linterp(tp), long_axis),
            ),
            // second point in sphere
            (true, false) => Line(
                build_vec(proj_line.linterp(tm), perp.linterp(tm), long_axis),
                line.1,
            ),
            // first point in sphere
            (false, true) => Line(
                line.0,
                build_vec(proj_line.linterp(tp), perp.linterp(tp), long_axis),
            ),
            // line segment contained within the circle (intersection points outside (0,1))
            (false, false) => line,
        })
}

#[test]
fn test_clip_line_cylinder() {
    use crate::vector::Vec3;
    let line = Line(Vec3::new(0.5, 0.6, 0.7), Vec3::new(-0.4, -0.8, -0.9));
    let clipped_line = clip_line_cylinder(line.clone(), 1.0, 1.0);
    println!("clipped line {:}", clipped_line.clone().unwrap());
    assert!(clipped_line.unwrap().is_close(&line));

    // this test is bad and it should feel bad (not how clipping in a cylinder works)
    // let line = Line(Vec3::new(-2.0, 0.5, 0.0), Vec3::new(2.0, 0.8, 0.0));
    // let clipped_line = clip_line_cylinder(line.clone(), 1.0, 1.0);
    // let expected_line = Line(Vec3::new(-1.0, 0.5, 0.0), Vec3::new(1.0, 0.8, 0.0));
    // println!("clipped line {:}", clipped_line.clone().unwrap());
    //assert!(clipped_line.unwrap().is_close(&expected_line));
}

#[test]
fn test_tube_clipping() {
    use crate::vector::Vec3;
    //let line = Line(Vec3::new(-0.3333, -1.0, -0.333), Vec3::new(-0.3333, 1.666, 0.333));
    let line = Line(Vec3::new(-1.0, 2.0, 1.0), Vec3::new(0.4, -1.0, 0.1));
    let clipped_line = clip_line_tube(line, 0.5);
    println!("{}", clipped_line.unwrap())
}

pub enum ReturnLines<V> {
    TwoLines(Line<V>, Line<V>),
    OneLine(Line<V>),
    NoLines,
}
impl<V> Iterator for ReturnLines<V> {
    type Item = Line<V>;

    fn next(&mut self) -> Option<Self::Item> {
        // TODO: this implementation is slow
        let out: Option<Self::Item>;
        // do the ol switcheroo
        (*self, out) = match std::mem::replace(self, Self::NoLines) {
            Self::NoLines => (Self::NoLines, None),
            Self::OneLine(line) => (Self::NoLines, Some(line)),
            Self::TwoLines(line_1, line_2) => (Self::OneLine(line_2), Some(line_1)),
        };
        out
    }
}
// TODO: to reduce allocation:
// create drawline buffer object
// give object a moving write index
// on each iteration, move index back to where we started, and rewrite
// see the vec truncate method
pub fn clip_line<V: VectorTrait>(
    line: Line<V>,
    boundaries: &[&ConvexBoundarySet<V>],
    write_lines: &mut Vec<Line<V>>,
    scratch: &mut Vec<Line<V>>,
) {
    let len = write_lines.len();
    write_lines.push(line);
    let mut new_len = write_lines.len();
    for convex_boundary_set in boundaries {
        scratch.clear();
        scratch.extend(write_lines[len..new_len].iter().cloned());
        write_lines.truncate(len);
        write_lines.extend(
            scratch
                .iter_mut()
                .flat_map(|line| clip_line_convex(line.clone(), convex_boundary_set)),
        );
        new_len = write_lines.len();

        //len = new_len;
    }
}

// TODO: robustly cover edge cases
pub fn clip_line_convex<V: VectorTrait>(
    line: Line<V>,
    boundary_set: &ConvexBoundarySet<V>,
) -> ReturnLines<V> {
    //if no boundaries, return original line
    let boundaries = &boundary_set.0;
    if boundaries.is_empty() {
        return ReturnLines::OneLine(line);
    }
    let Line(p0, p1) = line;

    let (mut a, mut b) = (0.0 as Field, 1.0 as Field);

    let (mut p0_all_safe, mut p1_all_safe) = (false, false);

    for boundary in boundaries {
        let n = boundary.normal;
        let th = boundary.threshold;

        let (p0n, p1n) = (p0.dot(n), p1.dot(n));
        let (p0_safe, p1_safe) = (p0n > th, p1n > th); // using > rather than >= here seems to reduce rogue lines

        if p0_safe && p1_safe {
            a = 0.0;
            b = 1.0;
            p0_all_safe = true;
            p1_all_safe = true;
            break;
        }
        if p0_safe && !p1_safe {
            let t_intersect = (p0n - th) / (p0n - p1n);
            a = a.max(t_intersect);
        }
        if !p0_safe && p1_safe {
            let t_intersect = (p0n - th) / (p0n - p1n);
            b = b.min(t_intersect);
        }
        p0_all_safe = p0_all_safe || p0_safe;
        p1_all_safe = p1_all_safe || p1_safe;
    }
    //both endpoints visible
    if p0_all_safe && p1_all_safe {
        //return two lines if we've intersected the shape
        //using b <= 1 rather than b < 1 seems to reduce rogue lines
        if a > 0.0 && b <= 1.0 {
            return ReturnLines::TwoLines(
                Line(p0, V::linterp(p0, p1, a)),
                Line(V::linterp(p0, p1, b), p1),
            );
        } else {
            //return entire line if we haven't intersected the shape
            return ReturnLines::OneLine(line);
        }
    }
    if p0_all_safe && !p1_all_safe {
        return ReturnLines::OneLine(Line(p0, V::linterp(p0, p1, a)));
    }
    if !p0_all_safe && p1_all_safe {
        return ReturnLines::OneLine(Line(V::linterp(p0, p1, b), p1));
    }
    //if neither point is visible, don't draw the line
    ReturnLines::NoLines
}

//consider using parallel joins here
// TODO: reduce vec pushing + allocation (~10% of runtime)
pub fn clip_draw_lines<'a, V: VectorTrait + 'a, I>(
    start_lines: &[DrawLine<V>],
    write_lines: &mut Vec<DrawLine<V>>,
    scratch: &mut Scratch<Line<V>>,
    clip_states_in_front: I,
)
//where for<'a> &'a I : std::iter::Iterator<Item=&'a ShapeClipState<V>>
where
    I: std::iter::Iterator<Item = &'a ShapeClipState<V>>,
{
    scratch.0.clear();
    scratch
        .0
        .extend(start_lines.iter().map(|dl| dl.line.clone()));

    let boundaries: Vec<&ConvexBoundarySet<V>> = clip_states_in_front
        .flat_map(|clip_state| (!clip_state.transparent).then_some(&clip_state.boundaries))
        .flat_map(|v| v.iter())
        .collect();

    scratch.2.clear();
    for opt_draw_line in &scratch.0 {
        clip_line(
            opt_draw_line.clone(),
            &boundaries,
            &mut scratch.2,
            &mut scratch.1,
        );
    }
    write_lines.extend(scratch.2.iter().map(|line| DrawLine {
        line: line.clone(),
        color: WHITE,
    }));
}

#[derive(Debug, Clone)]
pub enum Separation {
    Unknown,
    NoFront,
    S1Front,
    S2Front,
}
#[derive(Debug, Clone, Copy)]
pub enum Separator<V> {
    Unknown,
    Normal {
        normal: V,
        thresh_min: Field,
        thresh_max: Field,
        invert: bool,
    },
}
impl<V: VectorTrait> Separator<V> {
    pub fn apply(&self, origin: &V) -> Separation {
        match *self {
            Separator::Unknown => Separation::Unknown,
            Separator::Normal {
                normal,
                thresh_min,
                thresh_max,
                invert,
            } => {
                let dot_val = origin.dot(normal);
                if dot_val < thresh_min {
                    match invert {
                        false => Separation::S1Front,
                        true => Separation::S2Front,
                    }
                } else if dot_val > thresh_max {
                    match invert {
                        false => Separation::S2Front,
                        true => Separation::S1Front,
                    }
                } else {
                    Separation::NoFront
                }
            }
        }
    }
}
//use bounding spheres to find cases where shapes
//are not in front of others
//another function basically copied from
//John McIntosh (urticator.net)
pub fn dynamic_separate<V: VectorTrait>(
    bball1: &BBall<V>,
    bball2: &BBall<V>,
    origin: &V,
) -> Separation {
    let normal = bball1.pos - bball2.pos;
    let d = normal.norm();
    let (r1, r2) = (bball1.radius, bball2.radius);
    if d <= r1 + r2 {
        return Separation::Unknown;
    }

    let ratio = r1 / (r1 + r2);
    let dist1 = d * ratio;
    let reg1 = bball1.pos - normal * ratio;
    let reg1 = *origin - reg1;

    let adj = reg1.dot(normal) / d;
    let neg = r1 - dist1;
    let pos = d - r2 - dist1;
    if adj >= neg && adj <= pos {
        return Separation::NoFront;
    }

    let hyp2 = reg1.dot(reg1);
    let adj2 = adj * adj;
    let opp2 = hyp2 - adj2;

    let rcone = r1 / dist1;
    if opp2 >= hyp2 * rcone * rcone {
        return Separation::NoFront;
    }
    match adj > 0.0 {
        true => Separation::S2Front,
        false => Separation::S1Front,
    }
}

pub fn normal_separate<V: VectorTrait>(
    in_front1: InFrontArg<V>,
    in_front2: InFrontArg<V>,
    normal: &V,
) -> Separator<V> {
    const OVERLAP: Field = 1e-6;

    let nmin1 = in_front1
        .shape
        .verts
        .iter()
        .map(|v| v.dot(*normal))
        .fold(Field::NAN, Field::min);
    let nmax1 = in_front1
        .shape
        .verts
        .iter()
        .map(|v| v.dot(*normal))
        .fold(Field::NAN, Field::max);
    let nmin2 = in_front2
        .shape
        .verts
        .iter()
        .map(|v| v.dot(*normal))
        .fold(Field::NAN, Field::min);
    let nmax2 = in_front2
        .shape
        .verts
        .iter()
        .map(|v| v.dot(*normal))
        .fold(Field::NAN, Field::max);

    if nmin2 - nmax1 >= -OVERLAP {
        return Separator::Normal {
            normal: *normal,
            thresh_min: nmax1,
            thresh_max: nmin2,
            invert: true, //this is the opposite from source material, but works
        };
    }
    if nmin1 - nmax2 >= -OVERLAP {
        return Separator::Normal {
            normal: *normal,
            thresh_min: nmax2,
            thresh_max: nmin1,
            invert: false, //again, opposite from source material
        };
    }

    Separator::Unknown
}
pub fn separate_between_centers<V: VectorTrait>(
    in_front1: InFrontArg<V>,
    in_front2: InFrontArg<V>,
) -> Separator<V> {
    let normal = in_front2.bball.pos - in_front1.bball.pos;
    const EPSILON: Field = 1e-6;
    if normal.dot(normal) > EPSILON {
        normal_separate(in_front1, in_front2, &normal)
    } else {
        Separator::Unknown
    }
}
#[allow(dead_code)]
pub fn print_in_front(in_front: &[Vec<bool>]) {
    for row in in_front.iter() {
        println!();
        for val in row.iter() {
            print!(
                "{}, ",
                match val {
                    true => "1",
                    false => "0",
                }
            );
        }
    }
    println!();
}

#[allow(dead_code)]
pub fn test_dyn_separate<V: VectorTrait>(bballs: &[BBall<V>], origin: &V) {
    use colored::*;
    for (i, bball1) in bballs.iter().enumerate() {
        println!();
        for (j, bball2) in bballs.iter().enumerate() {
            if i != j {
                let sep = dynamic_separate(bball1, bball2, origin);
                let symb = match sep {
                    Separation::NoFront => "_".black(),
                    Separation::Unknown => "U".purple(),
                    Separation::S2Front => "2".yellow(),
                    Separation::S1Front => "1".white(),
                };
                print!("{}, ", symb)
            } else {
                print!("_, ")
            }
        }
    }
    println!();
}
