use specs::prelude::*;
use specs::{Component};
use crate::vector::{VectorTrait, MatrixTrait, VecIndex, Field, rotation_matrix, Vec4};

pub trait Transformable<V: VectorTrait> {
    fn with_transform(mut self, transformation: Transform<V>) -> Self
        where Self: std::marker::Sized {
        self.transform(transformation);
        self
    }
    fn transform(&mut self, transformation: Transform<V>);
    fn translate(&mut self, pos: V) {
        self.transform(Transform::pos(pos))
    }
    fn rotate(&mut self, axis1: VecIndex, axis2: VecIndex, angle: Field) {
        self.transform(Transform::identity().with_rotation(axis1, axis2, angle))
    }
    fn scale(&mut self, scale: Scaling<V>) {
        self.transform(Transform::new(None, Some(scale.get_mat())))
    }
    fn with_translation(mut self, pos: V) -> Self
        where Self: std::marker::Sized {
        self.translate(pos); self
    }
    fn with_rotation(mut self, axis1: VecIndex, axis2: VecIndex, angle: Field) -> Self
        where Self: std::marker::Sized {
        self.rotate(axis1, axis2, angle); self
    }
    fn with_rotation_about(self, axis1: VecIndex, axis2: VecIndex, angle: Field, point: V) -> Self
        where Self: std::marker::Sized {
        self.with_transform(Transform::identity().with_rotation_about(axis1, axis2, angle,point))
    }
    fn with_scale(mut self, scale: Scaling<V>) -> Self
        where Self: std::marker::Sized {
        self.scale(scale);
        self
    }
}

#[derive(Copy,Clone, Debug)]
pub enum Scaling<V: VectorTrait> {
    Scalar(Field),
    Vector(V),
}
impl<V: VectorTrait> Scaling<V> {
    fn unit() -> Self {
        Self::Scalar(1.0)
    }
    fn scale_vec(&self, vec: V) -> V {
        match self {
            Self::Scalar(s) => vec*(*s),
            Self::Vector(ref v) => v.elmt_mult(vec)
        }
    }
    fn compose(&self, rhs: &Self) -> Self {
        match (self, rhs) {
            (Self::Scalar(ref s1),Self::Scalar(ref s2)) => Self::Scalar(*s1**s2),
            (Self::Vector(ref v1),Self::Scalar(ref s2)) => Self::Vector(*v1**s2),
            (Self::Scalar(ref s1), Self::Vector(ref v2)) => Self::Vector(*v2**s1),
            (Self::Vector(_), Self::Vector(ref v2)) => Self::Vector(self.scale_vec(*v2)),
        }
    }
    pub(crate) fn get_vec(&self) -> V {
        match self {
            Scaling::Scalar(s) => V::ones() * *s,
            Scaling::Vector(v) => *v
        }
    }
    pub(crate) fn get_mat(&self) -> V::M {
        V::M::diag(self.get_vec())
    }
}

#[derive(Clone,Copy)]
// There is no proper subgroup of GL(n) generated by rotations + axis scaling (ortho + diag matrices)
// Proof: we can write any real matrix A as U H, where U in O(n) and H is symmetric.
// We can write H = V.T D V for some diagonal D, ortho V. So A = U V.T D V, a composition of ortho +
// diag matrices (I probably could have just used SVD)

// we could also create a more restrictive trait + struct RigidTransform, that requires the matrix A to be
// orthogonal. This might help us where the code assumes "frame" is orthogonal

// todo: there is a nice way to "compose" rotations and scalings, see blender
pub struct Transform<V: VectorTrait>{
    pub pos: V,
    pub frame: V::M,
}
impl<V: VectorTrait> Component for Transform<V> {
    type Storage = FlaggedStorage<Self, VecStorage<Self>>;
}
// todo figure out how to "snap" transforms to e.g. integer scales, deg of rotation, grid pos

// TODO improve performance by creating fewer structs? or does the compiler do that
impl<V: VectorTrait> Transform<V> {
    pub fn identity() -> Self {
        Self{
            pos: V::zero(),
            frame: V::M::id(),
        }
    }
    pub fn pos(pos: V) -> Self {
        let mut new = Transform::identity();
        new.pos = pos;
        new
    }
    pub fn new(maybe_pos: Option<V>, maybe_frame: Option<V::M>) -> Self {
        Self {
            pos: maybe_pos.unwrap_or(V::zero()),
            frame: maybe_frame.unwrap_or(V::M::id()),
        }
    }
    pub fn translate(&mut self, pos_delta: V) {
        self.pos = self.pos + pos_delta
    }

    pub fn rotate(&mut self, axis1: VecIndex, axis2: VecIndex, angle: Field) {
        let rot: Transform<V> = Transform::new(
            None,
            Some(rotation_matrix(
                self.frame[axis1],
                self.frame[axis2],
                Some(angle)
            ))
        );
        //self.frame = rot.frame.dot(self.frame)
        self.frame = rot.frame.dot(self.frame)
    }

    //todo update
    pub fn rotate_about(&mut self, axis1: VecIndex, axis2: VecIndex, angle: Field, pos: V) {
        let rot_mat = rotation_matrix(self.frame[axis1], self.frame[axis2], Some(angle));
        self.frame = self.frame.dot(rot_mat);
        self.pos = self.pos - rot_mat * pos
    }
    pub fn scale(&mut self, scale: Scaling<V>) {
        self.compose(Transform::new(None, Some(scale.get_mat())))
    }
    pub fn decompose_rotation_scaling(&self) -> (V::M, Scaling<V>) {
        // i tried using normalize_get_norm + unzip but rust hates me
        let cols: Vec<V> = self.frame.transpose().get_rows();
        let norms: Vec<Field> = cols.iter().map(|v|v.norm()).collect();
        //for n in norms.iter() { println!{":: {}", n}}
        (
            V::M::from_vec_of_vecs(
                &self.frame.transpose().get_rows().iter().zip(norms.iter()).map(|(v,n)| *v / *n).collect()
            ).transpose(),
            Scaling::Vector(V::from_iter(norms.iter()))
        )
    }
    pub fn unshear(&self) -> Transform<V> {
        let (rotation, scaling) = self.decompose_rotation_scaling();
        Transform::new(Some(self.pos), Some(rotation.dot(scaling.get_mat())))
    }

    pub fn transform_vec(&self, &vec: &V) -> V {
        self.frame * vec + self.pos
    }
    pub fn set_transform(&mut self, transform: Transform<V>) {
        self.pos = transform.pos;
        self.frame = transform.frame;
    }
    //FORMERLY transformations T1 v = R1 S1 v + p1 and T2 composed as
    //T1 T2 v = R1 R2 S1 S2 v + (p1 + p2)
    // NOW T1 = A1 v + p1 and T2 compose as affine transformations:
    // T1 T2 v = T1 (A2 v + p2) = A1 (A2 v + p2) + p1 = (A1 A2) v + (A1 p2 + p1)


    pub fn apply_self_on_left(&mut self, transformation: Transform<V>) {
        let other = transformation;
        self.pos = self.pos + self.frame * other.pos;
        self.frame = self.frame.dot(other.frame);
    }
    pub fn apply_self_on_right(&mut self, transformation: Transform<V>) {
        let other = transformation;
        self.pos = other.pos + other.frame * self.pos;
        self.frame = other.frame.dot(self.frame);
    }
    pub fn compose(&mut self, transformation: Transform<V>) {
        self.apply_self_on_left(transformation)
    }
    pub fn with_transform(mut self, transformation: Transform<V>) -> Self {
        self.compose(transformation); self
    }
    pub fn with_translation(mut self, pos_delta: V) -> Self {
        self.translate(pos_delta); self
    }
    pub fn with_rotation(mut self, axis1: VecIndex, axis2: VecIndex, angle: Field) -> Self {
        self.rotate(axis1, axis2, angle); self
    }
    pub fn with_rotation_about(mut self, axis1: VecIndex, axis2: VecIndex, angle: Field, point: V) -> Self {
        self.rotate_about(axis1, axis2, angle, point); self
    }
    pub fn with_scale(mut self, scaling: Scaling<V>) -> Self {
        self.scale(scaling); self
    }

}

#[allow(unused)]
#[test]
fn test_decompose() {
    use crate::vector::{Vec4, Mat4};
    let s = Scaling::Vector(Vec4::new(2.0, 3.0, 5.0, 7.0));
    let rot_mat = Mat4::from_arr(
        &[[-0.69214412,  0.44772088,  0.55884119, -0.09043814],
            [-0.19507629, -0.72900476,  0.2438655 , -0.60911979],
            [ 0.34303542, -0.2792939 ,  0.73238738,  0.51761989],
            [ 0.6043248 ,  0.43599655,  0.30304269, -0.59402329]]
    );
    let transform = Transform::new(
        Some(Vec4::zero()),
        Some(rot_mat.dot(s.get_mat()))
    );
    let (rot_mat_recon,s_recon) = transform.decompose_rotation_scaling();
    println!("{}", transform.frame.transpose());
    println!("{}", match s_recon {Scaling::Vector(v) => v, Scaling::Scalar(f) => Vec4::zero()});
    println!("{}", rot_mat_recon)
}

