use super::Vec4;
use crate::vector::{VectorTrait,MatrixTrait,VecIndex,Field};
use std::ops::{Add,Sub,Mul,Index};
use std::fmt;

//column major

#[derive(Copy,Clone)]
pub struct Mat4(Vec4,Vec4,Vec4,Vec4);

impl Mat4{
  pub fn from_vecs(v0 : Vec4, v1 : Vec4, v2 : Vec4, v3 : Vec4) -> Mat4 {
    Mat4::from_arr(&[
      *v0.get_arr(),
      *v1.get_arr(),
      *v2.get_arr(),
      *v3.get_arr(),
      ])
  }
  pub fn from_arr(arr : &[[Field ; 4] ; 4]) -> Mat4
  {
    Mat4(
      Vec4::from_arr(&arr[0]),
      Vec4::from_arr(&arr[1]),
      Vec4::from_arr(&arr[2]),
      Vec4::from_arr(&arr[3]),
      )
  }
}
impl Add<Mat4> for Mat4 {
  type Output = Mat4;
  
  fn add(self, rhs: Self) -> Mat4 {
    self.zip_map_els(rhs,|a,b| a + b)
    }
}

impl Sub<Mat4> for Mat4 {
  type Output = Mat4;
  
  fn sub(self, rhs: Self) -> Mat4 {
    self.zip_map_els(rhs,|a,b| a - b)
  }
}

impl Mul<Field> for Mat4 {
  type Output = Mat4;
  
  fn mul(self, rhs: Field) -> Mat4 {
    self.map_els(|m_ij| m_ij * rhs)
  }
}

impl Mul<Vec4> for Mat4 {
  type Output = Vec4;
  
  fn mul(self, rhs: Vec4) -> Vec4 {
    Vec4::new(
      self[0].dot(rhs),
      self[1].dot(rhs),
      self[2].dot(rhs),
      self[3].dot(rhs)
      )
  }
}
impl Index<VecIndex> for Mat4 {
    type Output = Vec4;

    fn index(&self, i: VecIndex) -> &Self::Output {
        match i {
            0 | -4 => &self.0,
            1 | -3 => &self.1,
            2 | -2 => &self.2,
            3 | -1 => &self.3,
            _ => panic!("Invalid index {} for Mat4", i)
        }
    }
}

impl MatrixTrait<Vec4> for Mat4 {
 // type V = Vec4;
  type Arr = [[Field ; 4] ; 4 ];

  fn get_arr(&self) -> Self::Arr {
    [
    *self[0].get_arr(),
    *self[1].get_arr(),
    *self[2].get_arr(),
    *self[3].get_arr(),
    ]
  }

  fn map_els<F : Fn(Field) -> Field + Copy>(self, f : F) -> Self {
    Self::from_arr(&[
      *self[0].map(f).get_arr(),
      *self[1].map(f).get_arr(),
      *self[2].map(f).get_arr(),
      *self[3].map(f).get_arr()
      ])
  }
  fn zip_map_els<F : Fn(Field,Field) -> Field + Copy>(self, rhs : Self, f : F) -> Self {
    Self::from_arr(&[
        *self[0].zip_map(rhs[0],f).get_arr(),
        *self[1].zip_map(rhs[1],f).get_arr(),
        *self[2].zip_map(rhs[2],f).get_arr(),
        *self[3].zip_map(rhs[3],f).get_arr(),
        ])
  }
  // fn transpose(self) -> Mat4 {
  //   Mat4(Vec4::new((self[0])[0],(self[1])[0]),
  //   Vec4::new((self[0])[1],(self[1])[1]))
  // }
  fn outer(v1: Vec4, v2: Vec4) -> Mat4 {
    Mat4::from_vecs(
      v2 * v1[0],
      v2 * v1[1],
      v2 * v1[2],
      v2 * v1[3],
      )
  }
  fn id() -> Mat4 {
    Mat4::from_arr(&[
      [1.,0.,0.,0.],
      [0.,1.,0.,0.],
      [0.,0.,1.,0.],
      [0.,0.,0.,1.]
      ])
  }
  fn dot(self, rhs: Mat4) -> Mat4 {
    let mut arr : Self::Arr = [[0.0 ; 4] ; 4];
    for i in 0..4 {
      for j in 0..4 {
        for k in 0..4 {
          arr[i][j] += self[i as VecIndex][k]*rhs[k][j as VecIndex]
        }
      }
    }
    Self::from_arr(&arr)
  }

}

impl fmt::Display for Mat4 {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} \n {} \n {} \n {}",
          self[0],self[1],self[2],self[3])
    }
}