use crate::vector::{VectorTrait,Field,scalar_linterp};
use crate::geometry::{Line,Plane,SubFace,Face};

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

pub fn calc_boundaries<V>(faces :  Vec<Face<V>>,
    subfaces : Vec<SubFace>,
    origin : V) -> Vec<Plane<V>>
where V : VectorTrait
{
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
