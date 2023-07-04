use serde::{Deserialize, Serialize};

use super::vec3::Vec3;
use crate::vector::{Field, Mat2, MatrixTrait, Vec2, VecIndex, VectorTrait};
use std::fmt;
use std::ops::{Add, Index, Mul, Sub};
use std::slice::Iter;

//column major

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Mat3(Vec3, Vec3, Vec3);

impl Mat3 {
    pub fn from_vecs(v0: Vec3, v1: Vec3, v2: Vec3) -> Mat3 {
        Mat3::from_arr(&[*v0.get_arr(), *v1.get_arr(), *v2.get_arr()])
    }
    pub fn from_arr(arr: &[[Field; 3]; 3]) -> Mat3 {
        Mat3(
            Vec3::from_arr(&arr[0]),
            Vec3::from_arr(&arr[1]),
            Vec3::from_arr(&arr[2]),
        )
    }
}

impl Add<Mat3> for Mat3 {
    type Output = Mat3;

    fn add(self, rhs: Self) -> Mat3 {
        self.zip_map_els(rhs, |a, b| a + b)
    }
}

impl Sub<Mat3> for Mat3 {
    type Output = Mat3;

    fn sub(self, rhs: Self) -> Mat3 {
        self.zip_map_els(rhs, |a, b| a - b)
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
        Vec3::new(self[0].dot(rhs), self[1].dot(rhs), self[2].dot(rhs))
    }
}

impl Index<VecIndex> for Mat3 {
    type Output = Vec3;

    fn index(&self, i: VecIndex) -> &Self::Output {
        match i {
            0 => &self.0,
            1 | -2 => &self.1,
            2 | -1 => &self.2,
            _ => panic!("Invalid index {} for Mat3", i),
        }
    }
}

impl MatrixTrait<Vec3> for Mat3 {
    // type V = Vec3;
    type Arr = [[Field; 3]; 3];

    fn get_arr(&self) -> Self::Arr {
        [*self[0].get_arr(), *self[1].get_arr(), *self[2].get_arr()]
    }

    fn map_els<F: Fn(Field) -> Field + Copy>(self, f: F) -> Self {
        Self::from_arr(&[
            *self[0].map(f).get_arr(),
            *self[1].map(f).get_arr(),
            *self[2].map(f).get_arr(),
        ])
    }
    fn zip_map_els<F: Fn(Field, Field) -> Field + Copy>(self, rhs: Self, f: F) -> Self {
        Self::from_arr(&[
            *self[0].zip_map(rhs[0], f).get_arr(),
            *self[1].zip_map(rhs[1], f).get_arr(),
            *self[2].zip_map(rhs[2], f).get_arr(),
        ])
    }
    // fn transpose(self) -> Mat3 {
    //   Mat3(Vec3::new((self[0])[0],(self[1])[0]),
    //   Vec3::new((self[0])[1],(self[1])[1]))
    // }
    fn outer(v1: Vec3, v2: Vec3) -> Mat3 {
        Mat3::from_vecs(v2 * v1[0], v2 * v1[1], v2 * v1[2])
    }
    fn id() -> Mat3 {
        Mat3::from_arr(&[[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]])
    }
    fn diag(v: Vec3) -> Mat3 {
        Mat3::from_arr(&[[v[0], 0., 0.], [0., v[1], 0.], [0., 0., v[2]]])
    }
    fn dot(self, rhs: Mat3) -> Mat3 {
        let mut arr: Self::Arr = [[0.0; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                for k in 0..3 {
                    arr[i][j] += self[i as VecIndex][k] * rhs[k][j as VecIndex]
                }
            }
        }
        Self::from_arr(&arr)
    }
    fn from_vec_of_vecs(vecs: &[Vec3]) -> Self {
        Mat3::from_vecs(vecs[0], vecs[1], vecs[2])
    }
    fn transpose(&self) -> Mat3 {
        let a = self.get_arr();
        Mat3::from_arr(&[
            [a[0][0], a[1][0], a[2][0]],
            [a[0][1], a[1][1], a[2][1]],
            [a[0][2], a[1][2], a[2][2]],
        ])
    }
}

impl fmt::Display for Mat3 {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} \n {} \n {}", self[0], self[1], self[2])
    }
}
