pub mod bball;

use std::collections::{HashSet,HashMap};
use crate::player::Player;
use crate::vector::{VectorTrait,Field};

use crate::geometry::{Line,Plane};
use crate::draw::DrawLine;
use crate::components::{Transform,Shape};

use specs::prelude::*;
use specs::{Component,VecStorage};
use std::marker::PhantomData;

use self::bball::BBall;

pub struct ClipState<V : VectorTrait> {
    //pub in_front : Vec<Vec<bool>>,
    //pub separators : Vec<Vec<Separator<V>>>,
    //pub in_front : HashSet<(Entity,Entity)>, //needs to be cleared whenever # shapes changes
    //pub separators : HashMap<(Entity,Entity),Separator<V>>, //ditto
    //pub separations_debug : Vec<Vec<Separation>>, //don't need this, but is useful for debug
    pub clipping_enabled : bool,
    phantom : std::marker::PhantomData<V>,
}
//could alternatively hold in a hash map over (entity,entity) pairs


impl<V : VectorTrait> Default for ClipState<V> {
    fn default() -> Self {ClipState::new()}
}

impl<V : VectorTrait> ClipState<V> {
    pub fn new() -> Self {
        //let shapes : Vec<&Shape<V>> = (&read_shapes).join().collect();
        ClipState {
            //in_front : HashSet::new(),
            //separations_debug :vec![vec![Separation::Unknown ; shapes_len] ; shapes_len],
            //separators : HashMap::new(),
            clipping_enabled : true,
            phantom : std::marker::PhantomData::<V>,
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

pub struct InFrontSystem<V : VectorTrait>(pub PhantomData<V>);
impl<'a,V : VectorTrait> System<'a> for InFrontSystem<V> {
    type SystemData = (
        ReadStorage<'a,Shape<V>>,
        ReadStorage<'a, BBall<V>>,
        WriteStorage<'a, ShapeClipState<V>>,
        Entities<'a>,
        ReadStorage<'a, Transform<V>>,
        ReadExpect<'a, Player>
    );

    fn run(&mut self, (shape_data, bball_data, mut shape_clip_state,entities,transform,player) : Self::SystemData) {
        calc_in_front(&shape_data, &bball_data,&mut shape_clip_state,&entities,&transform.get(player.0).unwrap().pos);
    }
}


//i've avoiding double mutable borrowing here by passing the entire shape_clip_states to calc_in_front_pair
//a disadvantage here is that we have no guarantee that the processed entities have the ShapeClipState component
//and that we have to iterate over all entities with the Shape component, instead of just those with both Shape and ShapeClipState
//but for now, every shape has a ShapeClipState.
pub fn calc_in_front<V : VectorTrait>(
        read_shapes : & ReadStorage<Shape<V>>,
        read_bballs: &ReadStorage<BBall<V>>,
        shape_clip_states : &mut WriteStorage<ShapeClipState<V>>,
        entities : &Entities,
        origin : &V,
    ) {
    //collect a vec of references to shapes
    //let shapes : Vec<&Shape<V>> = (& read_shapes).join().collect();
    //loop over unique pairs
    for (shape1, bball1, e1) in (read_shapes, read_bballs, &*entities).join() {
        for (shape2, bball2, e2) in (read_shapes, read_bballs, &*entities).join().filter(|(_sh,_bb,e)| *e > e1) {
            calc_in_front_pair(
                InFrontArg{shape : &shape1, bball: &bball1, entity : e1},
                InFrontArg{shape : &shape2, bball: &bball2, entity : e2},
                shape_clip_states,
                origin
                )
        }
    }
}

#[derive(Component)]
#[storage(VecStorage)]
pub struct ShapeClipState<V : VectorTrait> {
    pub in_front : HashSet<Entity>,
    pub separators : HashMap<Entity,Separator<V>>,
    pub boundaries : Vec<Plane<V>>,
    pub transparent: bool,
}
impl<V : VectorTrait> Default for ShapeClipState<V> {
   fn default() -> Self {
        Self{
            in_front : HashSet::new(),
            separators : HashMap::new(),
            boundaries : Vec::new(),
            transparent : false,
        }
    }
}
impl<V : VectorTrait> ShapeClipState<V> {
    pub fn in_front_debug(world : &World) -> String {
        let mut outstr = "In front debug \n".to_string();
        for (i,state) in world.read_storage::<ShapeClipState<V>>().join().enumerate() {
            outstr = format!("{}entity {}",outstr,i);
            outstr = format!("{} \n {}",outstr,state.this_in_front_debug());
        }
        outstr
    }
    pub fn this_in_front_debug(&self) -> String{
        use itertools::Itertools;
        let mut outstr = "".to_string();
         for e in self.in_front.iter().sorted() {
             outstr = format!("{} {} ",outstr,e.id());
         }
         outstr = format!("{}\n", outstr);
         outstr
    }
}
impl<V : VectorTrait> ShapeClipState<V> {
    pub fn remove(&mut self, e : &Entity) {
        self.in_front.remove(e);
        self.separators.remove(e);
    }
}
#[derive(Clone,Copy)]
pub struct InFrontArg<'a, V : VectorTrait>{
    shape : &'a Shape<V>,
    bball: &'a BBall<V>,
    entity : Entity,
}

pub fn calc_in_front_pair<'a,V :VectorTrait>(a : InFrontArg<'a,V>, b : InFrontArg<'a,V>,
    shape_clip_states : &mut WriteStorage<ShapeClipState<V>>, origin : &V) {

    //try dynamic separation
    let mut sep_state = dynamic_separate(a.bball,b.bball,origin);
    let is_unknown = match sep_state {
        Separation::Unknown => true,
        _ => false
    };
    //if that's unsuccessful, try static separation
    if is_unknown {
        let a_clip_state = shape_clip_states.get_mut(a.entity).unwrap();
        //let mut a_separators = shape_clip_states.get_mut(a.entity).unwrap().separators;
        let maybe_sep = a_clip_state.separators.get_mut(&b.entity);

        //compute static separator if it hasn't been computed yet
        let sep = match maybe_sep {
            Some(s) => *s,
            None => {
                let s = separate_between_centers(a,b);
                a_clip_state.separators.insert(b.entity, s);
                s
            }
        };

        //determine separation state from separator
        sep_state = sep.apply(origin);
    };
    let new_vals = match sep_state {
        Separation::S1Front => (true,false),
        Separation::S2Front => (false,true),
        Separation::NoFront => (false,false),
        Separation::Unknown => (true,true)
    };
    {
    let a_clip_state = shape_clip_states.get_mut(a.entity).unwrap();
    match new_vals.0 {
        true => a_clip_state.in_front.insert(b.entity),
        false => a_clip_state.in_front.remove(&b.entity),
    };}
    {
    let b_clip_state = shape_clip_states.get_mut(b.entity).unwrap();
    match new_vals.1 {
        true => b_clip_state.in_front.insert(a.entity),
        false => b_clip_state.in_front.remove(&a.entity),
    };}

}


pub fn clip_line_plane<V>(line : Line<V>, plane : &Plane<V>, small_z : Field) -> Option<Line<V>>
where V : VectorTrait {
    let Line(p0,p1)= line;

    let n = plane.normal; let th = plane.threshold + small_z;

    let (p0n, p1n) = (p0.dot(n), p1.dot(n));
    let (p0_safe,p1_safe) = (p0n >= th, p1n >= th);
    //both points behind
    if !p0_safe && !p1_safe {
        return None;
    }
    //both points in front
    if p0_safe && p1_safe {
        return Some(line);
    }
    //otherwise only one of the vertices is behind the camera
    let t_intersect = (p0n - th)/(p0n - p1n);
    let intersect = V::linterp(p0,p1,t_intersect);
    if (!p0_safe) && p1_safe {
        Some(Line(intersect, p1))
    } else {
        Some(Line(p0, intersect))
    }

}
pub fn clip_line_cube<V : VectorTrait>(line : Line<V>, r : Field) -> Option<Line<V>> {
    //construct the d cube planes, normals facing in
    let planes_iter = (0..V::DIM).map(
        move |i| ([-1., 1.]).iter()
            .map(move |&sign| Plane{normal : V::one_hot(i)*sign, threshold : -r})
        )
        .flatten();
    //successively clip on each plane
    let mut clip_line = Some(line);
    for plane in planes_iter {
        clip_line = match clip_line {
            Some(line) => clip_line_plane(line,&plane,0.),
            None => None,
        }
    }
    clip_line
}
pub fn clip_line_sphere<V :VectorTrait>(line : Line<V>, r : Field) -> Option<Line<V>> {
    let v0 = line.0;
    let v1 = line.1;

    let v0_in_sphere = v0.dot(v0) < r * r;
    let v1_in_sphere = v1.dot(v1) < r * r;

    if v0_in_sphere && v1_in_sphere {
        return Some(line);
    }

    let intersect = crate::geometry::sphere_line_intersect(line, r);
    match &intersect {
        None => None,
        Some(ref iline) => {
            if !v0_in_sphere && !v1_in_sphere {
                intersect
            } else if !v0_in_sphere && v1_in_sphere {
                Some(Line(iline.0, v1))
            } else {
                Some(Line(v0, iline.1))
            }
        }
    }
    
}
pub enum ReturnLines<V : VectorTrait>
{
    TwoLines(Line<V>,Line<V>),
    OneLine(Line<V>),
    NoLines
}
pub fn clip_line<V : VectorTrait>(
    line : Line<V>,
    boundaries : &Vec<Plane<V>>
    ) -> ReturnLines<V> {
    //if no boundaries, return original line
    if boundaries.len() == 0 {
        return ReturnLines::OneLine(line)
    }
    let Line(p0,p1)= line;

    let (mut a,mut b) = (0.0 as Field,1.0 as Field);

    let (mut p0_all_safe, mut p1_all_safe) = (false,false);

    for boundary in boundaries {

        let n = boundary.normal;
        let th = boundary.threshold;

        let (p0n, p1n) = (p0.dot(n), p1.dot(n));
        let (p0_safe, p1_safe) = (p0n >= th, p1n >= th);

        if p0_safe && p1_safe {
            a = 0.0; b = 1.0;
            p0_all_safe = true; p1_all_safe = true;
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
        if a > 0.0 && b < 1.0 {
            return ReturnLines::TwoLines(
                Line(p0, V::linterp(p0,p1,a) ),
                Line(V::linterp(p0,p1,b), p1)
                )
        } else {
            //return entire line if we haven't intersected the shape
            return ReturnLines::OneLine(line)
        }
    }
    if p0_all_safe && !p1_all_safe {
        return ReturnLines::OneLine(Line(p0, V::linterp(p0,p1,a)))
    }
    if !p0_all_safe && p1_all_safe {
        return ReturnLines::OneLine(Line(V::linterp(p0,p1,b), p1))
    }
    //if neither point is visible, don't draw the line
    ReturnLines::NoLines
}

//consider using parallel joins here
pub fn clip_draw_lines<'a, V : VectorTrait,I : std::iter::Iterator<Item=&'a ShapeClipState<V>>>(
    lines : Vec<Option<DrawLine<V>>>,
    clip_states_in_front : I
    ) ->  Vec<Option<DrawLine<V>>>
{
    let mut clipped_lines = lines;


    // let clipping_shapes : Vec<&Shape<V>> = match shape_in_front {
    //     Some(in_fronts) => shapes.join().zip(in_fronts)
    //         .filter(|(_shape,&front)| front)
    //         .map(|(shape,_front)| shape).collect(),
    //     None => shapes.join().collect()
    // };
    for clipping_shape in clip_states_in_front {
        //compare pointers
        // let same_shape = match shape {
        //     Some(shape) => clipping_shape as *const _ == shape as *const _,
        //     None => false
        // };
        //let same_shape = clip_shape_index == shape_index;
        if !clipping_shape.transparent {
            //let mut additional_lines : Vec<Option<Line<V>>> = Vec::new();
            let mut new_lines : Vec<Option<DrawLine<V>>> = Vec::new();
            //would like to map in place here, with side effects
            //(creating additonal lines)
            //worst case, we could push back and forth between two Vecs
            //with capacities slightly greater than initial # lines
            //right now i just push on to a new Vec every time
            for opt_draw_line in clipped_lines.into_iter() {
                let new_line = match opt_draw_line {
                    Some(DrawLine{line,color}) => match clip_line(line, &clipping_shape.boundaries) {
                        ReturnLines::TwoLines(line0,line1) => {
                            //additional_lines.push(Some(line1)); //push extra lines on to other vector
                            new_lines.push(Some(DrawLine{line : line1,color}));
                            Some(DrawLine{line : line0,color})
                        }
                        ReturnLines::OneLine(line) => Some(DrawLine{line,color}),
                        ReturnLines::NoLines => None

                    }
                    None => None
                };
                new_lines.push(new_line);
            }
            clipped_lines = new_lines;
            //clipped_lines.append(&mut additional_lines);
        }
    }
    clipped_lines
}

#[derive(Debug,Clone)]
pub enum Separation {
    Unknown,
    NoFront,
    S1Front,
    S2Front
}
#[derive(Debug,Clone,Copy)]
pub enum Separator<V : VectorTrait> {
    Unknown,
    Normal{
        normal : V,
        thresh_min : Field, thresh_max : Field,
        invert : bool
    }
}
impl<V : VectorTrait> Separator<V> {
    pub fn apply(&self, origin : &V) -> Separation {
        match *self {
            Separator::Unknown => Separation::Unknown,
            Separator::Normal{normal, thresh_min, thresh_max, invert} => {
                let dot_val = origin.dot(normal);
                if dot_val < thresh_min {
                    match invert {
                        false => Separation::S1Front,
                        true => Separation::S2Front
                    }
                    
                } else {
                    if dot_val > thresh_max {
                        match invert {
                            false => Separation::S2Front,
                            true => Separation::S1Front
                        }
                    } else {
                        Separation::NoFront
                    }
                }
            }
        }
    }
}
//use bounding spheres to find cases where shapes
//are not in front of others
//another function basically copied from
//John McIntosh (urticator.net)
pub fn dynamic_separate<V : VectorTrait>(
    bball1 : &BBall<V>,
    bball2 : &BBall<V>,
    origin: &V) -> Separation {
    let normal = bball1.pos - bball2.pos;
    let d = normal.norm();
    let (r1,r2) = (bball1.radius,bball2.radius);
    if d <= r1 + r2 {
        return Separation::Unknown
    }

    let ratio = r1/(r1+r2);
    let dist1 = d*ratio;
    let reg1 = bball1.pos - normal*ratio;
    let reg1 = *origin - reg1;

    let adj = reg1.dot(normal)/d;
    let neg = r1 - dist1;
    let pos = d - r2 - dist1;
    if adj >= neg && adj <= pos {
        return Separation::NoFront
    }

    let hyp2 = reg1.dot(reg1);
    let adj2 = adj*adj;
    let opp2 = hyp2 - adj2;

    let rcone = r1/dist1;
    if opp2 >= hyp2*rcone*rcone {
        return Separation::NoFront
    }
    match adj > 0.0 {
        true => Separation::S2Front,
        false => Separation::S1Front
    }
}


pub fn normal_separate<V : VectorTrait>(
    in_front1: InFrontArg<V>, in_front2 : InFrontArg<V>, normal : &V
) -> Separator<V> {
    const OVERLAP : Field = 1e-6;

    let nmin1 = in_front1.shape.verts.iter().map(|v| v.dot(*normal)).fold(0./0., Field::min);
    let nmax1 = in_front1.shape.verts.iter().map(|v| v.dot(*normal)).fold(0./0., Field::max);
    let nmin2 = in_front2.shape.verts.iter().map(|v| v.dot(*normal)).fold(0./0., Field::min);
    let nmax2 = in_front2.shape.verts.iter().map(|v| v.dot(*normal)).fold(0./0., Field::max);

    if nmin2 - nmax1 >= -OVERLAP {
        return Separator::Normal{
            normal : *normal,
            thresh_min : nmax1, thresh_max : nmin2,
            invert : true //this is the opposite from source material, but works
        }
    }
    if nmin1 - nmax2 >= -OVERLAP {
        return Separator::Normal{
            normal : *normal,
            thresh_min : nmax2, thresh_max : nmin1,
            invert : false //again, opposite from source material
        }
    }


    return Separator::Unknown
}
pub fn separate_between_centers<V : VectorTrait>(
    in_front1: InFrontArg<V>, in_front2 : InFrontArg<V>
    ) -> Separator<V>
{
    let normal = in_front2.bball.pos - in_front1.bball.pos;
    const EPSILON : Field = 1e-6;
    if normal.dot(normal) > EPSILON {
        normal_separate(in_front1, in_front2, &normal)
    } else {
        Separator::Unknown
    }

}
#[allow(dead_code)]
pub fn print_in_front(in_front : &Vec<Vec<bool>>) {
    for row in in_front.iter() {
        println!("");
        for val in row.iter() {
            print!("{}, ",match val {
                true => "1",
                false => "0"
            });
        }
    }
    println!("");
}

pub fn test_dyn_separate<V : VectorTrait>(bballs: &Vec<BBall<V>>, origin : &V) {
    use colored::*;
    for (i,bball1) in bballs.iter().enumerate() {
        println!("");
        for (j, bball2) in bballs.iter().enumerate() {
            if i != j {
                let sep = dynamic_separate(bball1,bball2,origin);
                let symb = match sep {
                    Separation::NoFront => "_".black(),
                    Separation::Unknown => "U".purple(),
                    Separation::S2Front => "2".yellow(),
                    Separation::S1Front => "1".white(),
                };
                print!("{}, ",symb)
            } else {
                print!("_, ")
            }
        }
    }
    println!("");
}