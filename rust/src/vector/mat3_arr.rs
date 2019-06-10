use super::vec3::Vec3;
use crate::vector::{VectorTrait,MatrixTrait,VecIndex,Field};
use std::ops::{Add,Sub,Mul,Index};
use std::fmt;

//column major

#[derive(Copy,Clone)]
pub struct Mat3{arr : [[Field ; 3] ; 3]}

impl Mat3{
  pub fn from_vecs(v0 : Vec3, v1 : Vec3, v2 : Vec3) -> Mat3 {
    Mat3::from_arr(&[
      *v0.get_arr(),
      *v1.get_arr(),
      *v2.get_arr()
      ])
  }
  pub fn from_arr(arr : &[[Field ; 3] ; 3]) -> Mat3
  {
    Mat3{arr : *arr}
  }
}
impl Add<Mat3> for Mat3 {
  type Output = Mat3;
  
  fn add(self, rhs: Self) -> Mat3 {
    self.zip_map_els(rhs,|a,b| a + b)
    }
}

impl Sub<Mat3> for Mat3 {
  type Output = Mat3;
  
  fn sub(self, rhs: Self) -> Mat3 {
    self.zip_map_els(rhs,|a,b| a - b)
  }
}

impl Mul<Field> for Mat3 {
  type Output = Mat3;
  
  fn mul(self, rhs: Field) -> Mat3 {
    self.map_els(|m_ij| m_ij * rhs)
  }
}

impl Mul<Vec3> for Mat3 {
  type Output = Vec3;
  
  fn mul(self, rhs: Vec3) -> Vec3 {
    Vec3::new(
      self[0].dot(rhs),
      self[1].dot(rhs),
      self[2].dot(rhs)
      )
  }
}
impl Index<VecIndex> for Mat3 {
    type Output = Vec3;

    fn index(&self, i: VecIndex) -> &Self::Output {
        match i {
            0 => &Vec3::from_arr(&self.arr[0]),
            1 | -2 => &Vec3::from_arr(&self.arr[1]),
            2 | -1 => &Vec3::from_arr(&self.arr[1]),
            _ => panic!("Invalid index {} for Mat3", i)
        }
    }
}

impl MatrixTrait<Vec3> for Mat3 {
 // type V = Vec3;
  type Arr = [[Field ; 3] ; 3 ];

  fn get_arr(&self) -> &Self::Arr {
    &self.arr
  }

  fn map_els<F : Fn(Field) -> Field>(self, f : F) -> Self {
    Self::from_arr(&[
      *self[0].map(f).get_arr(),
      *self[1].map(f).get_arr(),
      *self[2].map(f).get_arr()
      ])
  }
  fn zip_map_els<F : Fn(Field,Field) -> Field>(self, rhs : Self, f : F) -> Self {
    Self::from_arr(&[
        *self[0].zip_map(rhs[0],f).get_arr(),
        *self[1].zip_map(rhs[1],f).get_arr(),
        *self[2].zip_map(rhs[1],f).get_arr()
        ])
  }
  // fn transpose(self) -> Mat3 {
  //   Mat3(Vec3::new((self[0])[0],(self[1])[0]),
  //   Vec3::new((self[0])[1],(self[1])[1]))
  // }
  fn outer(v1: Vec3, v2: Vec3) -> Mat3 {
    Mat3::from_vecs(v2 * v1[0], v2 * v1[1], v2 * v2[2])
  }
  fn id() -> Mat3 {
    Mat3::from_arr(&[
      [1.,0.,0.],
      [0.,1.,0.],
      [0.,0.,1.]
      ])
  }
  fn dot(self, rhs: Mat3) -> Mat3 {
    let mut arr : Self::Arr = [[0.0 ; 3] ; 3];
    for i in 0..3 {
      for j in 0..3 {
        for k in 0..3 {
          arr[i][j] += self[i as VecIndex][k]*rhs[k][j as VecIndex]
        }
      }
    }
    Self::from_arr(&arr)
  }

}

impl fmt::Display for Mat3 {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} \n {}", self[0],self[1])
    }
}