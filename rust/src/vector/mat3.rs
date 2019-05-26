use super::vec3::Vec3;
use crate::vector::{VectorTrait,MatrixTrait,VecIndex,Field};
use std::ops::{Add,Sub,Mul,Index};
use std::fmt;

#[derive(Copy,Clone)]
pub struct Mat3(pub Vec3,pub Vec3,pub Vec3);
impl Index<VecIndex> for Mat3 {
    type Output = Vec3;

    fn index(&self, i: VecIndex) -> &Self::Output {
        match i {
            0 => &self.0,
            1 => &self.1,
            2 => &self.2,
            _ => panic!("Invalid index {} for Mat3", i)
        }
    }
}
impl Mat3{
  pub fn new(c : &[Field; 9]) -> Mat3 {
    Mat3(Vec3::new(c[0],c[1],c[2]),Vec3::new(c[3],c[4],c[5]),Vec3::new(c[6],c[7],c[8]))
  }
}
impl Add<Mat3> for Mat3 {
  type Output = Mat3;
  
  fn add(self, rhs: Self) -> Mat3 {
  Mat3(self[0]+rhs[0],self[1]+rhs[1],self[2]+rhs.2)
  }
}

impl Sub<Mat3> for Mat3 {
  type Output = Mat3;
  
  fn sub(self, rhs: Self) -> Mat3 {
  Mat3(self[0]-rhs[0],self[1]-rhs[1],self[2]-rhs.2)
  }
}

impl Mul<Field> for Mat3 {
  type Output = Mat3;
  
  fn mul(self, rhs: Field) -> Mat3 {
    Mat3(self[0]*rhs,self[1]*rhs,self[2]*rhs)
  }
}

impl Mul<Vec3> for Mat3 {
  type Output = Vec3;
  
  fn mul(self, rhs: Vec3) -> Vec3 {
    Vec3::new(self[0].dot(rhs), self[1].dot(rhs), self[2].dot(rhs))
  }
}
impl MatrixTrait<Vec3> for Mat3 {
//  type V = Vec3;

  fn transpose(self) -> Mat3 {
    Mat3(Vec3::new(self[0][0],self[1][0],self[2][0]),
    Vec3::new(self[0][1],self[1][1],self[2][1]),
    Vec3::new(self[0][2],self[1][2],self[2][2]))
  }
  fn outer(v1: Vec3, v2: Vec3) -> Mat3 {
    Mat3(v2 * v1[0], v2 * v1[1], v2 * v1[2])
  }
  fn id() -> Mat3 {
    Mat3::new(&
      [1.,0.,0.,
      0.,1.,0.,
      0.,0.,1.])
  }
  fn dot(self, rhs: Mat3) -> Mat3 {
    let rhs_t = rhs.transpose();
    Mat3(self * (rhs_t)[0], self * (rhs_t)[1], self * (rhs_t)[2]).transpose()
  }
}

impl fmt::Display for Mat3 {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} \n {} \n {}", self[0],self[1],self[2])
    }
}