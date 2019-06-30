use crate::vector::{VectorTrait,Field,scalar_linterp};
use crate::geometry::{Line,Plane,SubFace,Face,Shape};
use crate::draw::DrawLine;
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

pub fn clip_draw_lines<V : VectorTrait>(
    lines : Vec<Option<DrawLine<V>>>,
    shapes: &Vec<Shape<V>>,
    shape_in_front : Option<&Vec<bool>>
    ) ->  Vec<Option<DrawLine<V>>>
{
    let mut clipped_lines = lines;
    let clipping_shapes : Vec<&Shape<V>> = match shape_in_front {
        Some(in_fronts) => shapes.iter().zip(in_fronts)
            .filter(|(_shape,&front)| front)
            .map(|(shape,_front)| shape).collect(),
        None => shapes.iter().collect()
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
#[derive(Debug)]
pub enum Separator {
    Unknown,
    NoFront,
    S1Front,
    S2Front
}
//use bounding spheres to find cases where shapes
//are not in front of others
//another function basically copied from
//John McIntosh (urticator.net)
pub fn dynamic_separate<V : VectorTrait>(
    shape1 : &Shape<V>,
    shape2 : &Shape<V>,
    origin: &V) -> Separator {
    let normal = *shape1.get_pos() - *shape2.get_pos();
    let d = normal.norm();
    let (r1,r2) = (shape1.radius,shape2.radius);
    if d <= r1 + r2 {
        return Separator::Unknown
    }

    let ratio = r1/(r1+r2);
    let dist1 = d*ratio;
    let reg1 = *shape1.get_pos() - normal*ratio;
    let reg1 = *origin - reg1;

    let adj = reg1.dot(normal)/d;
    let neg = r1 - dist1;
    let pos = d - r2 - dist1;
    if adj >= neg && adj <= pos {
        return Separator::NoFront
    }

    let hyp2 = reg1.dot(reg1);
    let adj2 = adj*adj;
    let opp2 = hyp2 - adj2;

    let rcone = r1/dist1;
    if opp2 >= hyp2*rcone*rcone {
        return Separator::NoFront
    }
    match adj > 0.0 {
        true => Separator::S2Front,
        false => Separator::S1Front
    }
}
pub fn init_in_front<V : VectorTrait>(
    shapes : &Vec<Shape<V>>) -> Vec<Vec<bool>> {
    vec![vec![false ; shapes.len()] ; shapes.len()]
}
pub fn calc_in_front<V : VectorTrait>(
    in_front : &mut Vec<Vec<bool>>,
    shapes : &Vec<Shape<V>>,
    origin : &V) {
    for i in 0..shapes.len() {
        for j in i+1 .. shapes.len() {
                let sep = dynamic_separate(&shapes[i],&shapes[j],origin);
                let new_vals = match sep {
                    Separator::S1Front => (true,false),
                    Separator::S2Front => (false,true),
                    Separator::NoFront => (false,false),
                    Separator::Unknown => (true,true)
                };
                in_front[i][j] = new_vals.0;
                in_front[j][i] = new_vals.1;
        }
    }
}
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
                    Separator::NoFront => "_".black(),
                    Separator::Unknown => "U".purple(),
                    Separator::S2Front => "2".yellow(),
                    Separator::S1Front => "1".white(),
                };
                print!("{}, ",symb)
            } else {
                print!("_, ")
            }
        }
    }
    println!("");
}