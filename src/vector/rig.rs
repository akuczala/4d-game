mod rig4;

use super::VecIndex;
use std::fmt::Display;
use std::ops::{Add,Mul,Index,IndexMut};

pub trait RigField : Display + Copy + Sync + Send + 'static + Add<Output=Self> + Mul<Self,Output=Self> {
  fn zero() -> Self;
  fn one() -> Self;
 }

pub trait Rig<T : RigField>: Copy + Display + Sync + Send + 'static +
 Add<Output=Self> + Mul<T,Output=Self> +
 Index<VecIndex,Output=T> + IndexMut<VecIndex>
 //+ std::iter::Sum
 {

  type Arr;
  type SubR : Rig<T>;

  const DIM : VecIndex;

  fn from_arr(arr : &Self::Arr) -> Self;
  fn get_arr(&self) -> &Self::Arr;
  //ideally, I'd be able to implement this here by constrainting Arr to be iterable
  //could we use IntoIterator?
  fn iter<'a>(&'a self) -> std::slice::Iter<'a,T>;
  fn map<F : Fn(T) -> T>(self, f : F) -> Self;
  fn zip_map<F : Fn(T,T) -> T>(self, rhs : Self, f : F) -> Self;
  fn fold<F : Fn(T, T) -> T>(self, init : Option<T>, f : F) -> T;

  fn dot(self, rhs: Self) -> T;

  fn zero() -> Self {
    Self::constant(T::zero())
  }
  fn ones() -> Self {
    Self::constant(T::zero())
  }
  fn constant(a : T) -> Self;
  //fn project(&self) -> Self::SubR;

}
//impl<T : RigField, R> Add for R where R : Rig<T> {
// impl<T : RigField, R> Add for R where R : Rig<T> {
//   type Output = R;
  
//   fn add(self, rhs: Self) -> R {
//     self.zip_map(rhs,|a,b| a+b)
//   }
// }
