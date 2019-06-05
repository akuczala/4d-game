use crate::vector::{VectorTrait,Field,scalar_linterp};
use crate::geometry::{Line,Plane,SubFace,Face,Shape};

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

    if p0_all_safe && p1_all_safe {
        //return two lines if we've intersected the shape
        if a > 0.0 && b < 1.0 {
            return ReturnLines::TwoLines(
                Line(p0, V::linterp(p0,p1,a) ),
                Line(V::linterp(p0,p1,b), p1)
                )
        } else {
            return ReturnLines::OneLine(line)
        }
    }
    if p0_all_safe && !p1_all_safe {
        return ReturnLines::OneLine(Line(p0, V::linterp(p0,p1,a)))
    }
    if !p0_all_safe && p1_all_safe {
        return ReturnLines::OneLine(Line(V::linterp(p0,p1,b), p1))
    }
    ReturnLines::NoLines
}

pub fn clip_lines<V : VectorTrait>(
    mut lines : Vec<Option<Line<V>>>,
    shape : &Shape<V>,
    clipping_shapes: &Vec<Shape<V>>
    ) ->  Vec<Option<Line<V>>>
{
    let mut clipped_lines = lines;
    for clipping_shape in clipping_shapes {
        //compare pointers
        let same_shape = clipping_shape as *const _ != shape as *const _;
        if same_shape && !clipping_shape.transparent {
            let mut additional_lines : Vec<Option<Line<V>>> = Vec::new();
            //would like to map in place here, with side effects
            //(creating additonal lines)
            //worst case, we could push back and forth between two Vecs
            //with capacities slightly greater than initial # lines
            for opt_line in clipped_lines.iter_mut() {
                *opt_line = match *opt_line {
                    Some(line) => match clip_line(line, &clipping_shape.boundaries) {
                        ReturnLines::TwoLines(line0,line1) => {
                            additional_lines.push(Some(line1)); //push extra lines on to other vector
                            Some(line0)
                        }
                        ReturnLines::OneLine(line) => Some(line),
                        ReturnLines::NoLines => None

                    }
                    None => None
                }
            }
            clipped_lines.append(&mut additional_lines);
        }
    }
    clipped_lines
}
pub fn calc_boundaries<V : VectorTrait>(faces :  Vec<Face<V>>,
    subfaces : Vec<SubFace>,
    origin : V) -> Vec<Plane<V>> {
    let mut boundaries : Vec<Plane<V>> = Vec::new();
    for subface in subfaces {
        let face1 = &faces[subface.faceis[0]];
        let face2 = &faces[subface.faceis[1]];
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
    assert!(k1*k2 < 0.0);

    let t = k1/(k1 - k2);

    let n3 = V::linterp(n1, n2, t);
    let th3 = scalar_linterp(th1, th2, t);

    Plane{normal : n3, threshold: th3}
}
