
use crate::engine::Player;
use crate::vector::{VectorTrait,Field,scalar_linterp};
use crate::geometry::{Line,Plane,SubFace,Face,Shape};
use crate::draw::DrawLine;

use specs::prelude::*;
use std::marker::PhantomData;

use crate::camera::Camera;

pub struct InFrontSystem<V : VectorTrait>(pub PhantomData<V>);
impl<'a,V : VectorTrait> System<'a> for InFrontSystem<V> {
    type SystemData = (Write<'a,ClipState<V>>,ReadStorage<'a,Shape<V>>,ReadStorage<'a,Camera<V>>,ReadExpect<'a,Player>);

    fn run(&mut self, (mut clip_state,shape_data,camera,player) : Self::SystemData) {
        calc_in_front(&mut clip_state,shape_data.as_slice(),&camera.get(player.0).unwrap().pos);
    }
}
pub fn calc_in_front<V : VectorTrait>(
        clip_state : &mut ClipState<V>,
        //read_shapes : ReadStorage<Shape<V>>,
        shapes : &[Shape<V>],
        origin : &V
    ) {
        //collect a vec of references to shapes
        //let shapes : Vec<&Shape<V>> = (& read_shapes).join().collect();
        //loop over unique pairs
        for i in 0..shapes.len() {
            for j in i+1 .. shapes.len() {
                //try dynamic separation
                let mut sep_state = dynamic_separate(&shapes[i],&shapes[j],origin);
                let is_unknown = match sep_state {
                    Separation::Unknown => true,
                    _ => false
                };
                //if that's unsuccessful, try static separation
                if is_unknown {
                    let sep = &mut clip_state.separators[i][j];
                    //compute static separator if it hasn't been computed yet
                    let needs_value = match sep {
                        Separator::Unknown => true,
                        _ => false
                    };
                    //right now tries only one static separator
                    if needs_value {
                        *sep = separate_between_centers(&shapes[i],&shapes[j]);
                    }
                    //determine separation state from separator
                    sep_state = sep.apply(origin);
                };
                let new_vals = match sep_state {
                    Separation::S1Front => (true,false),
                    Separation::S2Front => (false,true),
                    Separation::NoFront => (false,false),
                    Separation::Unknown => (true,true)
                };
                clip_state.in_front[i][j] = new_vals.0;
                clip_state.in_front[j][i] = new_vals.1;

                //debug
                clip_state.separations_debug[i][j] = sep_state;
            }
        }
    }

pub struct ClipState<V : VectorTrait> {
    pub in_front : Vec<Vec<bool>>,
    pub separators : Vec<Vec<Separator<V>>>,
    pub separations_debug : Vec<Vec<Separation>>, //don't need this, but is useful for debug
    pub clipping_enabled : bool,
}
impl<V : VectorTrait> Default for ClipState<V> {
    fn default() -> Self {ClipState::new(0)}
}

impl<V : VectorTrait> ClipState<V> {
    pub fn new(shapes_len : usize) -> Self {
        //let shapes : Vec<&Shape<V>> = (&read_shapes).join().collect();
        ClipState {
            in_front : vec![vec![false ; shapes_len] ; shapes_len],
            separations_debug :vec![vec![Separation::Unknown ; shapes_len] ; shapes_len],
            separators : vec![vec![Separator::Unknown ; shapes_len] ; shapes_len],
            clipping_enabled : true,
        }
    }
    #[allow(dead_code)]
    pub fn print_debug(&self) {
        for row in self.separations_debug.iter() {
            println!("");
            for val in row.iter() {
                    print!("{}, ",match val {
                        Separation::S1Front => "1",
                        Separation::S2Front => "2",
                        Separation::NoFront => "_",
                        Separation::Unknown => "U"
                    });
                }
            }
            println!("");
    }
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
pub fn clip_draw_lines<V : VectorTrait>(
    lines : Vec<Option<DrawLine<V>>>,
    shapes: &ReadStorage<Shape<V>>,
    shape_in_front : Option<&Vec<bool>>
    ) ->  Vec<Option<DrawLine<V>>>
{
    let mut clipped_lines = lines;
    let clipping_shapes : Vec<&Shape<V>> = match shape_in_front {
        Some(in_fronts) => shapes.join().zip(in_fronts)
            .filter(|(_shape,&front)| front)
            .map(|(shape,_front)| shape).collect(),
        None => shapes.join().collect()
    };
    for clipping_shape in clipping_shapes {
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
pub fn calc_boundaries<V : VectorTrait>(faces :  &Vec<Face<V>>,
    subfaces : &Vec<SubFace>,
    origin : V) -> Vec<Plane<V>> {

    let mut boundaries : Vec<Plane<V>> = Vec::new();

    for subface in subfaces {
        let face1 = &faces[subface.faceis.0];
        let face2 = &faces[subface.faceis.1];
        if face1.visible == !face2.visible {
            let boundary = calc_boundary(face1, face2, origin);
            boundaries.push(boundary);
        }
    }
    //visible faces are boundaries
    for face in faces {
        if face.visible {
            boundaries.push(Plane{
                normal : face.normal, threshold : face.threshold
            })
        }
    }
    boundaries
}

pub fn calc_boundary<V>(face1 : &Face<V>,
    face2 : &Face<V>,
    origin : V) -> Plane<V>
where V : VectorTrait
{
    let (n1,n2) = (face1.normal,face2.normal);
    let (th1,th2) = (face1.threshold, face2.threshold);

    //k1 and k2 must have opposite signs
    let k1 = n1.dot(origin) - th1;
    let k2 = n2.dot(origin) - th2;
    //assert!(k1*k2 < 0.0,"k1 = {}, k2 = {}",k1,k2);

    let t = k1/(k1 - k2);

    let n3 = V::linterp(n1, n2, t);
    let th3 = scalar_linterp(th1, th2, t);

    Plane{normal : n3, threshold: th3}
}
#[derive(Debug,Clone)]
pub enum Separation {
    Unknown,
    NoFront,
    S1Front,
    S2Front
}
#[derive(Debug,Clone)]
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
    shape1 : &Shape<V>,
    shape2 : &Shape<V>,
    origin: &V) -> Separation {
    let normal = *shape1.get_pos() - *shape2.get_pos();
    let d = normal.norm();
    let (r1,r2) = (shape1.radius,shape2.radius);
    if d <= r1 + r2 {
        return Separation::Unknown
    }

    let ratio = r1/(r1+r2);
    let dist1 = d*ratio;
    let reg1 = *shape1.get_pos() - normal*ratio;
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
    shape1 : &Shape<V>, shape2 : &Shape<V>, normal : &V
) -> Separator<V> {
    const OVERLAP : Field = 1e-6;

    let nmin1 = shape1.verts.iter().map(|v| v.dot(*normal)).fold(0./0., Field::min);
    let nmax1 = shape1.verts.iter().map(|v| v.dot(*normal)).fold(0./0., Field::max);
    let nmin2 = shape2.verts.iter().map(|v| v.dot(*normal)).fold(0./0., Field::min);
    let nmax2 = shape2.verts.iter().map(|v| v.dot(*normal)).fold(0./0., Field::max);

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
    shape1 : &Shape<V>, shape2 : &Shape<V>
    ) -> Separator<V>
{
    let normal = *shape2.get_pos() - *shape1.get_pos();
    const EPSILON : Field = 1e-6;
    if normal.dot(normal) > EPSILON {
        normal_separate(shape1, shape2, &normal)
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
pub fn test_dyn_separate<V : VectorTrait>(shapes : &Vec<Shape<V>>, origin : &V) {
    use colored::*;
    for (i,s1) in shapes.iter().enumerate() {
        println!("");
        for (j,s2) in shapes.iter().enumerate() {
            if i != j {
                let sep = dynamic_separate(s1,s2,origin);
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