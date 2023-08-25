use std::{
    fmt,
    ops::{Add, Index, Mul, Sub},
};

use serde::{Deserialize, Serialize};

use super::{vec1::Vec1, Field, MatrixTrait, VecIndex, VectorTrait, FROM_ITER_ERROR_MESSAGE};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Mat1(pub Vec1);
impl Mat1 {
    pub fn from_vecs(v0: Vec1) -> Mat1 {
        Mat1::from_arr(&[*v0.get_arr()])
    }
    pub fn from_arr(arr: &[[Field; 1]; 1]) -> Mat1 {
        Mat1(Vec1::from_arr(&arr[0]))
    }
}
impl Add<Mat1> for Mat1 {
    type Output = Mat1;

    fn add(self, rhs: Self) -> Mat1 {
        self.zip_map_els(rhs, |a, b| a + b)
    }
}

impl Sub<Mat1> for Mat1 {
    type Output = Mat1;

    fn sub(self, rhs: Self) -> Mat1 {
        self.zip_map_els(rhs, |a, b| a - b)
    }
}

impl Mul<Field> for Mat1 {
    type Output = Mat1;

    fn mul(self, rhs: Field) -> Mat1 {
        self.map_els(|m_ij| m_ij * rhs)
    }
}

impl Mul<Vec1> for Mat1 {
    type Output = Vec1;

    fn mul(self, rhs: Vec1) -> Vec1 {
        Vec1::new(self[0].dot(rhs))
    }
}
impl Index<VecIndex> for Mat1 {
    type Output = Vec1;

    fn index(&self, i: VecIndex) -> &Self::Output {
        match i {
            0 | -1 => &self.0,
            _ => panic!("Invalid index {} for Mat1", i),
        }
    }
}

impl FromIterator<Vec1> for Mat1 {
    fn from_iter<T: IntoIterator<Item = Vec1>>(iter: T) -> Self {
        let mut into_iter = iter.into_iter();
        Self(into_iter.next().expect(FROM_ITER_ERROR_MESSAGE))
    }
}

impl MatrixTrait<Vec1> for Mat1 {
    type Arr = [[Field; 1]; 1];

    fn get_arr(&self) -> Self::Arr {
        [*self[0].get_arr()]
    }
    fn from_vec_of_vecs(vecs: &[Vec1]) -> Self {
        Mat1::from_vecs(vecs[0])
    }

    fn map_els<F: Fn(Field) -> Field + Copy>(self, f: F) -> Self {
        Self::from_arr(&[*self[0].map(f).get_arr()])
    }
    fn zip_map_els<F: Fn(Field, Field) -> Field + Copy>(self, rhs: Self, f: F) -> Self {
        Self::from_arr(&[*self[0].zip_map(rhs[0], f).get_arr()])
    }
    fn outer(v1: Vec1, v2: Vec1) -> Mat1 {
        Mat1::from_vecs(v2 * v1[0])
    }
    fn id() -> Mat1 {
        Mat1(Vec1::new(1.0))
    }
    fn diag(v: Vec1) -> Mat1 {
        Mat1(v)
    }
    fn dot(self, rhs: Mat1) -> Mat1 {
        Self(Vec1::new(self.0[0] * rhs.0[0]))
    }
    fn transpose(&self) -> Mat1 {
        *self
    }
}

impl fmt::Display for Mat1 {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} \n {}", self[0], self[1])
    }
}
