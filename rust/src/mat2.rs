use crate::vec2::Vec2;
use crate::vector::{VectorTrait,MatrixTrait,VecIndex,Field};
use std::ops::{Add,Sub,Mul,Div,Index};
use std::fmt;

#[derive(Copy,Clone)]
pub struct Mat2(pub Vec2,pub Vec2);

impl Mat2{
  pub fn new(c : &[Field; 4]) -> Mat2 {
    Mat2(Vec2(c[0],c[1]),Vec2(c[2],c[3]))
  }
}
impl Add<Mat2> for Mat2 {
  type Output = Mat2;
  
  fn add(self, rhs: Self) -> Mat2 {
  Mat2(self.0+rhs.0,self.1+rhs.1)
  }
}

impl Sub<Mat2> for Mat2 {
  type Output = Mat2;
  
  fn sub(self, rhs: Self) -> Mat2 {
  Mat2(self.0-rhs.0,self.1-rhs.1)
  }
}

impl Mul<Field> for Mat2 {
  type Output = Mat2;
  
  fn mul(self, rhs: Field) -> Mat2 {
    Mat2(self.0*rhs,self.1*rhs)
  }
}

impl Mul<Vec2> for Mat2 {
  type Output = Vec2;
  
  fn mul(self, rhs: Vec2) -> Vec2 {
    Vec2(self.0.dot(rhs), self.1.dot(rhs))
  }
}
impl Index<VecIndex> for Mat2 {
    type Output = Vec2;

    fn index(&self, i: VecIndex) -> &Self::Output {
        match i {
            0 => &self.0,
            1 => &self.1,
            _ => panic!("Invalid index {} for Mat2", i)
        }
    }
}

impl MatrixTrait<Vec2> for Mat2 {
 // type V = Vec2;

  fn transpose(self) -> Mat2 {
    Mat2(Vec2((self.0).0,(self.1).0),
    Vec2((self.0).1,(self.1).1))
  }
  fn outer(v1: Vec2, v2: Vec2) -> Mat2 {
    Mat2(v2 * v1.0, v2 * v1.1)
  }
  fn id() -> Mat2 {
    Mat2::new(&
      [1.,0.,
      0.,1.])
  }
  //probably not correct
  fn dot(self, rhs: Mat2) -> Mat2 {
    let rhs_t = rhs.transpose();
    Mat2(self * (rhs_t).0, self * (rhs_t).1).transpose()
  }
}

impl fmt::Display for Mat2 {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} \n {}", self.0,self.1)
    }
}