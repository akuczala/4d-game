pub mod vec2; pub mod vec3; pub mod vec4;
pub mod mat2_tuple2; pub mod mat3_tuple2;
pub mod mat4_tuple2;
//pub mod vec4;
use fmt::Display;
use serde::{Serialize, Deserialize};
use std::ops::{Add, Sub, Mul, Div, Index, IndexMut, Neg, Range};
pub use vec2::Vec2;
pub use vec3::Vec3;
pub use vec4::Vec4;
pub use mat2_tuple2::Mat2;
pub use mat3_tuple2::Mat3;
pub use mat4_tuple2::Mat4;
use std::{array, fmt};
//use alga::linear::FiniteDimInnerSpace;
pub type VecIndex = i8; //i8
pub type Field = f32;

const EPSILON : Field = 0.0001;
pub use std::f32::consts::PI;
use std::slice::Iter;

pub fn is_close(a : Field, b : Field) -> bool {
    (a-b).abs() < EPSILON
}
pub fn scalar_linterp(a : Field, b : Field, t : Field) -> Field {
    a*(1.0-t) + b*t
}
pub fn linspace(min : Field, max : Field, n : usize) -> impl Iterator<Item=Field> {
    (0..n).map(move |i| (i as Field)/((n-1) as Field)).map(move |f| (1.-f)*min + f*max)
}

const FROM_ITER_ERROR_MESSAGE: &str = "Invalid index in from iter";
//consider using #![feature(associated_consts)]
//to define vector dimension (might not need to explicity use feature?)

// TODO: check if we really need all these bounds
pub trait VectorTrait: Copy + Display + std::fmt::Debug +
    Add<Output=Self> + Sub<Output=Self> + Neg<Output=Self> +
    Mul<Field,Output=Self> + Div<Field,Output=Self> +
    Index<VecIndex,Output=Field> + IndexMut<VecIndex>
    //+ std::iter::Sum
    {
        type M : MatrixTrait<Self>;
        type SubV: VectorTrait;
        type Arr;

        const DIM : VecIndex;

        fn from_arr(arr : &Self::Arr) -> Self;
        fn from_iter(iter: Iter<Field>) -> Self;
        fn get_arr(&self) -> &Self::Arr;
        //ideally, I'd be able to implement this here by constrainting Arr to be iterable
        //could we use IntoIterator?
        fn iter<'a>(&'a self) -> std::slice::Iter<'a,Field>;
        fn map<F : Fn(Field) -> Field>(self, f : F) -> Self;
        fn zip_map<F : Fn(Field,Field) -> Field>(self, rhs : Self, f : F) -> Self;
        fn fold<F : Fn(Field, Field) -> Field>(self, init : Option<Field>, f : F) -> Field;

        fn dot(self, rhs: Self) -> Field;
        fn norm_sq(self) -> Field {
            self.dot(self)
        }
        fn norm(self) -> Field {
            self.norm_sq().sqrt()
        }
        fn normalize_get_norm(self) -> (Self, Field) {(|n| (self / n, n))(self.norm())}
        fn normalize(self) -> Self {
            self/self.norm()
        }
        fn is_close(v1: Self, v2: Self) -> bool {
            (v1-v2).norm_sq() < EPSILON*EPSILON
        }
        fn zero() -> Self {
            Self::constant(0.0)
        }
        fn ones() -> Self {
            Self::constant(1.0)
        }
        fn constant(a : Field) -> Self;
        fn one_hot(i: VecIndex) -> Self{
            Self::M::id()[i]
        }
        fn project(&self) -> Self::SubV;
        fn unproject(v: Self::SubV) -> Self;
        fn linterp(v1: Self, v2: Self,x : Field) -> Self {
            v1*(1.-x) + v2*x
        }
        fn cross_product<I : std::iter::Iterator<Item=Self>>(points : I) -> Self;
        fn elmt_mult(&self, rhs: Self) -> Self {
            self.zip_map(rhs, |x,y| x*y)
        }
        fn random() -> Self {
			Self::ones().map(|_| rand::random())
		}
    }
//impl<T> Foo for T where T: Clone + Mul<i64> + Add<i64> + ... {}

//fn foo<C>() where i64: From<C>, C: Foo {}


pub fn barycenter<V: VectorTrait>(vlist : &Vec<V>) -> V
{
    vlist.iter().fold(V::zero(),|sum,val| sum + *val)/(vlist.len() as Field)
}

pub fn barycenter_iter<'a, V, I>(viter: I) -> V
where
    V: VectorTrait + 'a,
    I: Iterator<Item=&'a V>,
    
{
    let (sum, len) = viter.fold(
        (V::zero(), 0),
        |(sum,len),&val| (sum + val, len + 1));
    sum/(len as Field)
}

// TODO: check if we really need all these bounds
pub trait MatrixTrait<V : VectorTrait>: Display + Copy +
    Add<Output=Self> + Sub<Output=Self> 
//+ Mul<Field,Output=Self> + Mul<Self,Output=Self>
 + Mul<V,Output=V> //weirdly only the last Mul is remembered
 + Index<VecIndex,Output=V>
{
    type Arr;

    fn get_arr(&self) -> Self::Arr;
    fn map_els<F : Fn(Field) -> Field + Copy>(self, f : F) -> Self;
    fn zip_map_els<F : Fn(Field,Field) -> Field + Copy>(self, rhs : Self, f : F) -> Self;
    //fn transpose(self) -> Self;
    fn outer(v1: V, v2: V) -> Self;
    fn id() -> Self;
    fn diag(v: V) -> Self;
    fn dot(self, rhs : Self) -> Self;
    fn scale(self, rhs : Field) -> Self {
        self.map_els(|mij| mij*rhs)
    }
    // use vecs of V for now, but it would be faster to use iterators of &V (probably?)
    fn get_rows(&self) -> Vec<V> {
        // is it possible to do this without a lambda? i tried self.index
        (0..V::DIM).map(|i| self[i]).collect()
    }
    fn from_vec_of_vecs(vecs: &Vec<V>) -> Self;
    fn transpose(&self) -> Self;
    
}


pub trait Translatable<V : VectorTrait> {
    fn get_pos(&self) -> V;
    fn set_pos(&mut self, new_pos : V);
    fn with_pos(mut self, new_pos : V) -> Self
    where Self: std::marker::Sized  {
        self.set_pos(new_pos);
        self
    }
    fn translate(&mut self, dpos  : V) {
        self.set_pos(self.get_pos() + dpos);
    }
}
pub trait Rotatable<V : VectorTrait> {
    fn get_frame(&self) -> V::M;
    fn set_frame(&mut self, new_frame : V::M);
    fn set_frame_into(mut self, new_frame : V::M) -> Self
    where Self: std::marker::Sized  {
        self.set_frame(new_frame);
        self
    }
    fn rotate(&mut self, axis1: VecIndex, axis2: VecIndex, angle : Field) {
            let rot_mat = rotation_matrix(self.get_frame()[axis1],self.get_frame()[axis2],Some(angle));
            self.set_frame(self.get_frame().dot(rot_mat));
    }
}

pub fn rotation_matrix<V>(v1 : V, v2: V, th : Option<Field>)-> V::M
where V : VectorTrait
{
    let u = v1/v1.norm();
    let v = v2/v2.norm();
     let costh = match th {
     None => u.dot(v),
     Some(angle) => angle.cos()
 };
 let sinth = match th {
     None => (1.0 - (costh*costh).min(1.0)).sqrt(),
     Some(angle) => angle.sin()
 };

 let mut w = v - u * u.dot(v);
 if !V::is_close(w,V::zero()) {
    w = w.normalize();
 }

 let r1 = u*costh - w*sinth;
 let r2 = u*sinth + w*costh;

 V::M::id() + V::M::outer(u,r1-u) + V::M::outer(w,r2-w)
}
pub fn diagnostic<V>(v1: V, v2: V)
where V: VectorTrait
{
    let mat = rotation_matrix(v1,v2,None);
    println!("rot mat:{}",mat);
    //println!("{}",mat.dot(mat.transpose()));
    println!("correct rotation: {}",V::is_close(v2.normalize(),mat * v1.normalize()));
    println!("Difference: {}", v2.normalize() - mat * v1.normalize());
}

pub fn test_vectors() {
        let v1 = Vec2::new(1.0,2.0);
        let v2 = Vec2::new(3.0,4.0);
        diagnostic(v1,v2);

        let v1 = Vec3::new(1.0,2.0,3.0);
        let v2 = Vec3::new(4.0,5.0,6.0);
        diagnostic(v1,v2);

        let v1 = Vec4::new(1.0,2.0,3.0,4.0);
        let v2 = Vec4::new(5.0,6.0,7.0,8.0);
        diagnostic(v1,v2);

        println!("Test matrix mult 2d");
        let mat1 = Mat2::from_arr(&[[1.0,2.0],[3.0,4.0]]);
        let mat2 = Mat2::from_arr(&[[5.0,6.0],[7.0,8.0]]);
        println!("{}",mat1.dot(mat2));

        println!("Test matrix mult 3d");
        let mat1 = Mat3::from_arr(&[[1.0,2.0,3.0],[4.0,5.0,6.0],[7.0,8.0,9.0]]);
        let mat2 = Mat3::from_arr(&[[10.,11.,12.],[13.,14.,15.],[16.,17.,18.]]);
        println!("{}",mat1.dot(mat2));

        println!("Test matrix mult 3d");
        let mat1 = Mat4::from_arr(&[
            [1.0,2.0,3.0,4.0],
            [5.0,6.0,7.0,8.0],
            [9.0,0.1,1.1,1.2],
            [1.3,1.4,1.5,1.6]
            ]);
        let mat2 = Mat4::from_arr(&[
            [10.,11.,12.,13.],
            [14.,15.,16.,17.],
            [18.,19.,20.,21.],
            [22.,23.,24.,25.]
            ]);
        println!("{}",mat1.dot(mat2));

    }

#[test]
pub fn test_cross_product() {
    assert!(
        VectorTrait::is_close(VectorTrait::cross_product(
            vec![Vec2::new(2.,3.)].into_iter()
        ),Vec2::new(-3.,2.))
    );
    assert!(
        VectorTrait::is_close(VectorTrait::cross_product(
            vec![
            Vec3::new(2.,3.,5.),
            Vec3::new(7.,11.,13.)
        
    ].into_iter()),Vec3::new(-16.,9.,1.))
    );
    let cross4 = VectorTrait::cross_product(
            vec![
            Vec4::new(2.,3.,5.,7.),
            Vec4::new(11.,13.,17.,19.),
            Vec4::new(23.,29.,31.,37.)
        
    ].into_iter());
    println!("{}",cross4);
    assert!(VectorTrait::is_close(cross4,
        Vec4::new(160.,-120.,-90.,70.))
    );
    // a cross product of 3 vectors in a plane yields 0 in four dimensions
    assert!(
        VectorTrait::is_close(
            VectorTrait::cross_product(
                vec![
                    Vec4::new(1., 1., 0., 0.),
                    Vec4::new(1., 0., 0. ,0.),
                    Vec4::new(0., 1., 0., 0.),
                ].into_iter()
            ),
            Vec4::zero()
        )
    )
}

#[test]
fn test_linspace() {
    assert!(linspace(-2.5,2.5,9).zip(
        vec![-2.5  , -1.875, -1.25 , -0.625,  0.   ,  0.625,  1.25 ,  1.875, 2.5  ]
    ).all(|(a,b)| is_close(a,b)))
}
#[test]
fn test_barycenter() {
    use rand::{Rng, thread_rng};
    let n = 10;
    let mut rng = thread_rng();
    let mut vecs: Vec<Vec3> = vec![];
    let mut accum = Vec3::zero();
    for _ in 0..n {
        let new_vec = Vec3::new(rng.gen(),rng.gen(),rng.gen());
        accum = accum + new_vec;
        vecs.push(new_vec);
    }
    let expected_center = accum / (n as Field);
    let center = barycenter(&vecs);
    let center_from_iter = barycenter_iter(vecs.iter());
    assert!(
        Vec3::is_close(center,expected_center),
        "center={}, expected_center={}",center,expected_center
    );
    assert!(
        Vec3::is_close(center_from_iter,expected_center),
        "center_from_iter={}, expected_center={}",center_from_iter,expected_center
    );
}