use std::ops::{Add,Sub,Mul,Div,Index};
use std::fmt;
use crate::vector::{VecIndex,VectorTrait,Field};
use super::mat2::Mat2;

#[derive(Copy,Clone)]
pub struct Vec2(pub Field,pub Field);

impl Add<Vec2> for Vec2 {
  type Output = Vec2;
  
  fn add(self, rhs: Self) -> Vec2 {
    Vec2(self.0+rhs.0,self.1+rhs.1)
  }
}
impl Sub<Vec2> for Vec2 {
  type Output = Vec2;
  
  fn sub(self, rhs: Self) -> Vec2 {
    Vec2(self.0-rhs.0,self.1-rhs.1)
  }
}
impl Mul<Field> for Vec2 {
  type Output = Vec2;
  
  fn mul(self, rhs: Field) -> Vec2 {
    Vec2(self.0*rhs,self.1*rhs)
  }
}
impl Div<Field> for Vec2 {
  type Output = Vec2;
  
  fn div(self, rhs: Field) -> Vec2 {
    Vec2(self.0/rhs,self.1/rhs)
  }
}
impl VectorTrait for Vec2 {
  type M = Mat2;
  fn norm(self) -> Field {
    (self.0*self.0 + self.1*self.1).sqrt()
  }
  fn dot(self, rhs: Vec2) -> Field {
    (self.0*rhs.0 + self.1*rhs.1)
  }
  fn zero() -> Vec2{
    Vec2(0.,0.)
  }
  fn ones() -> Vec2{
    Vec2(1.,1.)
  }
}

impl fmt::Display for Vec2 {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({},{})", self.0,self.1)
    }
}
impl Index<VecIndex> for Vec2 {
    type Output = Field;

    fn index(&self, i: VecIndex) -> &Self::Output {
        match i {
            0 => &self.0,
            1 => &self.1,
            _ => panic!("Invalid index {} for Vec2", i)
        }
    }
}
