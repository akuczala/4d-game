use std::ops::{Add,Sub,Mul,Div,Index};
use std::fmt;
use crate::vector::{VecIndex,VectorTrait,Field};
use super::mat2::Mat2;

#[derive(Copy,Clone)]
pub struct Vec2{pub arr : [Field ; 2]}
impl Vec2 {
  pub fn new(v0 : Field, v1 : Field) -> Vec2
  {
    Vec2{arr : [v0,v1]}
  }
  pub fn from_arr(arr : &[Field ; 2]) -> Vec2
  {
    Vec2{arr : *arr}
  }

}
impl Index<VecIndex> for Vec2 {
    type Output = Field;

    fn index(&self, i: VecIndex) -> &Field {
        match i {
             0 => &self.get_arr()[0],
             1 => &self.get_arr()[1],
            -1 => &self.get_arr()[1],
            -2 => &self.get_arr()[0],
            _ => panic!("Invalid index {} for Vec2", i)
        }
    }
}
impl Add<Vec2> for Vec2 {
  type Output = Vec2;
  
  fn add(self, rhs: Self) -> Vec2 {
    Vec2::new(self[0]+rhs[0],self[1]+rhs[1])
  }
}
impl Sub<Vec2> for Vec2 {
  type Output = Vec2;
  
  fn sub(self, rhs: Self) -> Vec2 {
    Vec2::new(self[0]-rhs[0],self[1]-rhs[1])
  }
}
impl Mul<Field> for Vec2 {
  type Output = Vec2;
  
  fn mul(self, rhs: Field) -> Vec2 {
    Vec2::new(self[0]*rhs,self[1]*rhs)
  }
}
impl Div<Field> for Vec2 {
  type Output = Vec2;
  
  fn div(self, rhs: Field) -> Vec2 {
    Vec2::new(self[0]/rhs,self[1]/rhs)
  }
}
impl VectorTrait for Vec2 {
  type M = Mat2;

  //this should be Field but we have to implement VectorTrait
  //for Field
  type SubV = Vec2; 

  type Arr = [Field ; 2];

  const DIM : VecIndex = 2;
  
  fn get_arr(&self) -> &[Field ; 2]{
    &self.arr
  }

  fn norm(self) -> Field {
    (self[0]*self[0] + self[1]*self[1]).sqrt()
  }
  fn dot(self, rhs: Vec2) -> Field {
    (self[0]*rhs[0] + self[1]*rhs[1])
  }
  fn zero() -> Vec2{
    Vec2::new(0.,0.)
  }
  fn ones() -> Vec2{
    Vec2::new(1.,1.)
  }
  //should really return Field
  //I instead just throw away the second component
  fn project(&self) -> Vec2 {
    Vec2::new(self[0],0.0)
  }
}

impl fmt::Display for Vec2 {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({},{})", self[0],self[1])
    }
}
