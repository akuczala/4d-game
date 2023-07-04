use serde::{Deserialize, Serialize};

use super::Vec2;
use crate::vector::{Field, MatrixTrait, VecIndex, VectorTrait};
use std::fmt;
use std::ops::{Add, Index, Mul, Sub};
use std::slice::Iter;

//column major

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Mat2(Vec2, Vec2);

impl Mat2 {
    pub fn from_vecs(v0: Vec2, v1: Vec2) -> Mat2 {
        Mat2::from_arr(&[*v0.get_arr(), *v1.get_arr()])
    }
    pub fn from_arr(arr: &[[Field; 2]; 2]) -> Mat2 {
        Mat2(Vec2::from_arr(&arr[0]), Vec2::from_arr(&arr[1]))
    }
}

impl Add<Mat2> for Mat2 {
    type Output = Mat2;

    fn add(self, rhs: Self) -> Mat2 {
        self.zip_map_els(rhs, |a, b| a + b)
    }
}

impl Sub<Mat2> for Mat2 {
    type Output = Mat2;

    fn sub(self, rhs: Self) -> Mat2 {
        self.zip_map_els(rhs, |a, b| a - b)
    }
}

impl Mul<Field> for Mat2 {
    type Output = Mat2;

    fn mul(self, rhs: Field) -> Mat2 {
        self.map_els(|m_ij| m_ij * rhs)
    }
}

impl Mul<Vec2> for Mat2 {
    type Output = Vec2;

    fn mul(self, rhs: Vec2) -> Vec2 {
        Vec2::new(self[0].dot(rhs), self[1].dot(rhs))
    }
}

impl Index<VecIndex> for Mat2 {
    type Output = Vec2;

    fn index(&self, i: VecIndex) -> &Self::Output {
        match i {
            0 => &self.0,
            1 | -1 => &self.1,
            _ => panic!("Invalid index {} for Mat2", i),
        }
    }
}

impl MatrixTrait<Vec2> for Mat2 {
    // type V = Vec2;
    type Arr = [[Field; 2]; 2];

    fn get_arr(&self) -> Self::Arr {
        [*self[0].get_arr(), *self[1].get_arr()]
    }
    fn from_vec_of_vecs(vecs: &Vec<Vec2>) -> Self {
        Mat2::from_vecs(vecs[0], vecs[1])
    }

    fn map_els<F: Fn(Field) -> Field + Copy>(self, f: F) -> Self {
        Self::from_arr(&[*self[0].map(f).get_arr(), *self[1].map(f).get_arr()])
    }
    fn zip_map_els<F: Fn(Field, Field) -> Field + Copy>(self, rhs: Self, f: F) -> Self {
        Self::from_arr(&[
            *self[0].zip_map(rhs[0], f).get_arr(),
            *self[1].zip_map(rhs[1], f).get_arr(),
        ])
    }
    // fn transpose(self) -> Mat2 {
    //   Mat2(Vec2::new((self[0])[0],(self[1])[0]),
    //   Vec2::new((self[0])[1],(self[1])[1]))
    // }
    fn outer(v1: Vec2, v2: Vec2) -> Mat2 {
        Mat2::from_vecs(v2 * v1[0], v2 * v1[1])
    }
    fn id() -> Mat2 {
        Mat2::from_arr(&[[1., 0.], [0., 1.]])
    }
    fn diag(v: Vec2) -> Mat2 {
        Mat2::from_arr(&[[v[0], 0.], [0., v[1]]])
    }
    fn dot(self, rhs: Mat2) -> Mat2 {
        let mut arr: Self::Arr = [[0.0; 2]; 2];
        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    arr[i][j] += self[i as VecIndex][k] * rhs[k][j as VecIndex]
                }
            }
        }
        Self::from_arr(&arr)
    }
    fn transpose(&self) -> Mat2 {
        let a = self.get_arr();
        Mat2::from_arr(&[[a[0][0], a[1][0]], [a[0][1], a[1][1]]])
    }
}

impl fmt::Display for Mat2 {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} \n {}", self[0], self[1])
    }
}
