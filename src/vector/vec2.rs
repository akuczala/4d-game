use serde::{Deserialize, Serialize};
use std::fmt;
use std::iter::Sum;
use std::ops::{Add, Div, Index, IndexMut, Mul, Neg, Sub};

use super::vec1::Vec1;
use super::Mat2;
use crate::vector::{Field, VecIndex, VectorTrait, FROM_ITER_ERROR_MESSAGE};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Vec2([Field; 2]);
impl Vec2 {
    pub fn new(v0: Field, v1: Field) -> Vec2 {
        Vec2([v0, v1])
    }
}
impl Index<VecIndex> for Vec2 {
    type Output = Field;

    fn index(&self, i: VecIndex) -> &Field {
        match i {
            0 => &self.get_arr()[0],
            1 => &self.get_arr()[1],
            -1 => &self.get_arr()[1],
            -2 => &self.get_arr()[0],
            _ => panic!("Invalid index {} for Vec2", i),
        }
    }
}
impl IndexMut<VecIndex> for Vec2 {
    fn index_mut(&mut self, index: VecIndex) -> &mut Self::Output {
        match index {
            0 | -2 => &mut self.0[0],
            1 | -1 => &mut self.0[1],
            _ => panic!("Invalid index {} for Vec2", index),
        }
    }
}

impl Add<Vec2> for Vec2 {
    type Output = Vec2;

    fn add(self, rhs: Self) -> Vec2 {
        self.zip_map(rhs, |a, b| a + b)
    }
}
impl Sub<Vec2> for Vec2 {
    type Output = Vec2;

    fn sub(self, rhs: Self) -> Vec2 {
        self.zip_map(rhs, |a, b| a - b)
    }
}
impl Neg for Vec2 {
    type Output = Vec2;

    fn neg(self) -> Vec2 {
        Vec2::zero() - self
    }
}

impl Mul<Field> for Vec2 {
    type Output = Vec2;

    fn mul(self, rhs: Field) -> Vec2 {
        //Vec2::new(self[0]*rhs,self[1]*rhs)
        self.map(|v_i| v_i * rhs)
    }
}
impl Div<Field> for Vec2 {
    type Output = Vec2;

    fn div(self, rhs: Field) -> Vec2 {
        self * (1.0 / rhs)
    }
}

impl Sum for Vec2 {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.reduce(|x, y| x + y).unwrap_or(Self::zero())
    }
}

impl FromIterator<Field> for Vec2 {
    fn from_iter<T: IntoIterator<Item = Field>>(iter: T) -> Self {
        let mut into_iter = iter.into_iter();
        Vec2::new(
            into_iter.next().expect(FROM_ITER_ERROR_MESSAGE),
            into_iter.next().expect(FROM_ITER_ERROR_MESSAGE),
        )
    }
}

impl VectorTrait for Vec2 {
    type M = Mat2;

    type SubV = Vec1;

    type Arr = [Field; 2];

    const DIM: VecIndex = 2;

    fn from_arr(arr: &Self::Arr) -> Self {
        Self(*arr)
    }

    fn get_arr(&self) -> &[Field; 2] {
        &self.0
    }
    fn iter(&self) -> std::slice::Iter<Field> {
        self.get_arr().iter()
    }
    fn map<F: Fn(Field) -> Field>(self, f: F) -> Self {
        Vec2::new(f(self[0]), f(self[1]))
    }
    fn zip_map<F: Fn(Field, Field) -> Field>(self, rhs: Self, f: F) -> Self {
        Vec2::new(f(self[0], rhs[0]), f(self[1], rhs[1]))
    }
    fn fold<F: Fn(Field, Field) -> Field>(self, init: Option<Field>, f: F) -> Field {
        let val0 = match init {
            Some(ival) => f(ival, self[0]),
            None => self[0],
        };
        f(val0, self[1])
    }
    fn dot(self, rhs: Vec2) -> Field {
        self[0] * rhs[0] + self[1] * rhs[1]
    }

    fn constant(a: Field) -> Vec2 {
        Vec2::new(a, a)
    }
    //should really return Field
    //I instead just throw away the second component
    fn project(&self) -> Vec1 {
        Vec1::new(self[0])
    }
    fn unproject(v: Vec1) -> Vec2 {
        Vec2::new(v[0], 0.0)
    }
    fn cross_product<I: std::iter::Iterator<Item = Self>>(mut vecs_iter: I) -> Self {
        let v0 = vecs_iter.next().expect("No vecs given to 2d cross product");
        if let Some(_v1) = vecs_iter.next() {
            panic!("2D cross product given more than 1 vec");
        }
        Vec2::new(-v0[1], v0[0]) //right handed
    }
}

impl fmt::Display for Vec2 {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({},{})", self[0], self[1])
    }
}
