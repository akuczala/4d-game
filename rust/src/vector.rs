pub mod vec2; pub mod vec3;
pub mod mat2; pub mod mat3;

use fmt::Display;
use std::ops::{Add,Sub,Mul,Div,Index,Neg};
pub use vec2::Vec2;
pub use vec3::Vec3;
pub use mat2::Mat2;
pub use mat3::Mat3;
use std::fmt;
//use alga::linear::FiniteDimInnerSpace;
pub type VecIndex = i8;
pub type Field = f32;

const EPSILON : Field = 0.0001;
pub use std::f32::consts::PI;
pub fn is_close(a : Field, b : Field) -> bool {
  (a-b).abs() < EPSILON
}
pub fn scalar_linterp(a : Field, b : Field, t : Field) -> Field {
  a*(1.0-t) + b*t
}
//consider using #![feature(associated_consts)]
//to define vector dimension (might not need to explicity use feature?)
pub trait VectorTrait: Copy + Display +
 Add<Output=Self> + Sub<Output=Self> + Neg<Output=Self> +
 Mul<Field,Output=Self> + Div<Field,Output=Self> +
 Index<VecIndex,Output=Field>
 //+ std::iter::Sum
 {
  type M : MatrixTrait<Self>;
  type SubV: VectorTrait;
  type Arr;

  const DIM : VecIndex;

  fn get_arr(&self) -> &Self::Arr;

  fn dot(self, rhs: Self) -> Field;
  fn norm_sq(self) -> Field {
    self.dot(self)
  }
  fn norm(self) -> Field {
    self.norm_sq().sqrt()
  }
  fn normalize(self) -> Self {
    self/self.norm()
  }
  fn is_close(v1: Self, v2: Self) -> bool {
    (v1-v2).norm_sq() < EPSILON*EPSILON
  }
  fn zero() -> Self;
  fn ones() -> Self;
  fn one_hot(i: VecIndex) -> Self{
    Self::M::id()[i]
  }
  fn project(&self) -> Self::SubV;
  fn linterp(v1: Self, v2: Self,x : Field) -> Self {
    v1*(1.-x) + v2*x
  }
}

//impl<T> Foo for T where T: Clone + Mul<i64> + Add<i64> + ... {}

//fn foo<C>() where i64: From<C>, C: Foo {}


pub fn barycenter<V>(vlist : Vec<V>) -> V
where V : VectorTrait
{
  vlist.iter().fold(V::zero(),|sum,val| sum + *val)/(vlist.len() as Field)
}
pub fn barycenter_iter<V>(viter : &mut std::slice::Iter<V>) -> V
where V : VectorTrait
{
  viter.fold(V::zero(),|sum,val| sum + *val)/(viter.len() as Field)
}

pub trait MatrixTrait<V>: Display + Copy + Add<Output=Self> + Sub<Output=Self> 
//+ Mul<Field,Output=Self> + Mul<Self,Output=Self>
 + Mul<V,Output=V> //weirdly only the last Mul is remembered
 + Index<VecIndex,Output=V>
{
  fn transpose(self) -> Self;
  fn outer(v1: V, v2: V) -> Self;
  fn id() -> Self;
  fn dot(self,rhs: Self) -> Self; //matrix multiplication
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

 let w = v - u * u.dot(v);
 let w = w.normalize();

 let r1 = u*costh - w*sinth;
 let r2 = u*sinth + w*costh;

 V::M::id() + V::M::outer(u,r1-u) + V::M::outer(w,r2-w)
}
pub fn diagnostic<V>(v1: V, v2: V)
where V: VectorTrait
{
  let mat = rotation_matrix(v1,v2,None);
  println!("rot mat:{}",mat);
  println!("{}",mat.dot(mat.transpose()));
  println!("correct rotation: {}",V::is_close(v2.normalize(),mat * v1.normalize()));
}

pub fn test_vectors() {
    let v1 = Vec2::new(1.0,2.0);
    let v2 = Vec2::new(3.0,4.0);
    diagnostic(v1,v2);

    let v1 = Vec3::new(1.0,2.0,3.0);
    let v2 = Vec3::new(4.0,5.0,6.0);
    diagnostic(v1,v2);

    println!("Test matrix mult 2d");
    let mat1 = Mat2::new(&[[1.0,2.0],[3.0,4.0]]);
    let mat2 = Mat2::new(&[[5.0,6.0],[7.0,8.0]]);
    println!("{}",mat1.dot(mat2));
    println!("Test matrix mult 3d");
    let mat1 = Mat3::new(&[[1.0,2.0,3.0],[4.0,5.0,6.0],[7.0,8.0,9.0]]);
    let mat2 = Mat3::new(&[[10.,11.,12.],[13.,14.,15.],[16.,17.,18.]]);
    println!("{}",mat1.dot(mat2));
  }