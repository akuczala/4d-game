use std::ops::{Add,Sub,Mul,Div,Index};
use std::fmt;
use crate::vector::{VecIndex,VectorTrait,Field,Vec2};
use super::mat3::Mat3;

#[derive(Copy,Clone)]
pub struct Vec3{arr: [Field ; 3]}
impl Vec3 {
  pub fn new(v0 : Field, v1 : Field, v2 : Field) -> Vec3
  {
    Vec3{arr : [v0,v1,v2]}
  }
  pub fn from_arr(arr : &[Field ; 3]) -> Vec3
  {
    Vec3{arr : *arr}
  }
}
impl Index<VecIndex> for Vec3 {
    type Output = Field;

    fn index(&self, i: VecIndex) -> &Self::Output {
        match i {
             0 => &self.get_arr()[0],
             1 => &self.get_arr()[1],
             2 => &self.get_arr()[2],
            -1 => &self.get_arr()[2],
            -2 => &self.get_arr()[1],
            -3 => &self.get_arr()[0],
            _ => panic!("Invalid index {} for Vec3", i)
        }
    }
}
impl Add<Vec3> for Vec3 {
  type Output = Vec3;
  
  fn add(self, rhs: Self) -> Vec3 {
  Vec3::new(self[0]+rhs[0],self[1]+rhs[1],self[2]+rhs[2])
  }
}

impl Sub<Vec3> for Vec3 {
  type Output = Vec3;
  
  fn sub(self, rhs: Self) -> Vec3 {
  Vec3::new(self[0]-rhs[0],self[1]-rhs[1],self[2]-rhs[2])
  }
}

impl Mul<Field> for Vec3 {
  type Output = Vec3;
  
  fn mul(self, rhs: Field) -> Vec3 {
    Vec3::new(self[0]*rhs,self[1]*rhs,self[2]*rhs)
  }
}

impl Div<Field> for Vec3 {
  type Output = Vec3;
  
  fn div(self, rhs: Field) -> Vec3 {
    Vec3::new(self[0]/rhs,self[1]/rhs,self[2]/rhs)
  }
}


impl VectorTrait for Vec3 {
  type M = Mat3;
  type SubV = Vec2;
  type Arr = [Field ; 3];

  const DIM : VecIndex = 3;

  fn get_arr(&self) -> &[Field ; 3]{
    &self.arr
  }

  fn norm(self) -> Field {
    (self[0]*self[0] + self[1]*self[1] + self[2]*self[2]).sqrt()
  }
  fn dot(self, rhs: Vec3) -> Field {
    (self[0]*rhs[0] + self[1]*rhs[1] + self[2]*rhs[2])
  }
  fn zero() -> Vec3{
    Vec3::new(0.,0.,0.)
  }
  fn ones() -> Vec3{
    Vec3::new(1.,1.,1.)
  }
  fn project(&self) -> Vec2 {
    Vec2::new(self[0],self[1])
  }
}

impl fmt::Display for Vec3 {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({},{},{})", self[0],self[1],self[2])
    }
}