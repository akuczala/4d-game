use std::ops::{Add,Sub,Mul,Div,Index};
use std::fmt;
use crate::vector::{VecIndex,VectorTrait,Field};
use crate::mat3::Mat3;

#[derive(Copy,Clone)]
pub struct Vec3(pub Field,pub Field,pub Field);

impl Add<Vec3> for Vec3 {
  type Output = Vec3;
  
  fn add(self, rhs: Self) -> Vec3 {
  Vec3(self.0+rhs.0,self.1+rhs.1,self.2+rhs.2)
  }
}

impl Sub<Vec3> for Vec3 {
  type Output = Vec3;
  
  fn sub(self, rhs: Self) -> Vec3 {
  Vec3(self.0-rhs.0,self.1-rhs.1,self.2-rhs.2)
  }
}

impl Mul<Field> for Vec3 {
  type Output = Vec3;
  
  fn mul(self, rhs: Field) -> Vec3 {
    Vec3(self.0*rhs,self.1*rhs,self.2*rhs)
  }
}

impl Div<Field> for Vec3 {
  type Output = Vec3;
  
  fn div(self, rhs: Field) -> Vec3 {
    Vec3(self.0/rhs,self.1/rhs,self.2/rhs)
  }
}


impl VectorTrait for Vec3 {
  type M = Mat3;
  fn norm(self) -> Field {
    (self.0*self.0 + self.1*self.1 + self.2*self.2).sqrt()
  }
  fn dot(self, rhs: Vec3) -> Field {
    (self.0*rhs.0 + self.1*rhs.1 + self.2*rhs.2)
  }
  fn zero() -> Vec3{
    Vec3(0.,0.,0.)
  }
  fn ones() -> Vec3{
    Vec3(1.,1.,1.)
  }
}

impl fmt::Display for Vec3 {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({},{},{})", self.0,self.1,self.2)
    }
}
impl Index<VecIndex> for Vec3 {
    type Output = Field;

    fn index(&self, i: VecIndex) -> &Self::Output {
        match i {
            0 => &self.0,
            1 => &self.1,
            2 => &self.2,
            _ => panic!("Invalid index {} for Vec3", i)
        }
    }
}