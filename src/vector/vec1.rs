use std::{
    fmt,
    ops::{Add, Div, Index, IndexMut, Mul, Neg, Sub},
    slice::Iter,
};

use serde::{Deserialize, Serialize};

use super::{mat1::Mat1, Field, MatrixTrait, VecIndex, VectorTrait, FROM_ITER_ERROR_MESSAGE};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Vec1([Field; 1]);

impl Vec1 {
    pub fn new(v0: Field) -> Vec1 {
        Vec1([v0])
    }
}
impl Index<VecIndex> for Vec1 {
    type Output = Field;

    fn index(&self, i: VecIndex) -> &Field {
        match i {
            0 | -1 => &self.get_arr()[0],
            _ => panic!("Invalid index {} for Vec1", i),
        }
    }
}
impl IndexMut<VecIndex> for Vec1 {
    fn index_mut(&mut self, index: VecIndex) -> &mut Self::Output {
        match index {
            0 | -1 => &mut self.0[0],
            _ => panic!("Invalid index {} for Vec1", index),
        }
    }
}

impl Add<Vec1> for Vec1 {
    type Output = Vec1;

    fn add(self, rhs: Self) -> Vec1 {
        self.zip_map(rhs, |a, b| a + b)
    }
}
impl Sub<Vec1> for Vec1 {
    type Output = Vec1;

    fn sub(self, rhs: Self) -> Vec1 {
        self.zip_map(rhs, |a, b| a - b)
    }
}
impl Neg for Vec1 {
    type Output = Vec1;

    fn neg(self) -> Vec1 {
        Vec1::zero() - self
    }
}

impl Mul<Field> for Vec1 {
    type Output = Vec1;

    fn mul(self, rhs: Field) -> Vec1 {
        //Vec1::new(self[0]*rhs,self[1]*rhs)
        self.map(|v_i| v_i * rhs)
    }
}
impl Div<Field> for Vec1 {
    type Output = Vec1;

    fn div(self, rhs: Field) -> Vec1 {
        self * (1.0 / rhs)
    }
}
impl VectorTrait for Vec1 {
    type M = Mat1;

    //this should be Field but we have to implement VectorTrait
    //for Field
    type SubV = Vec1;

    type Arr = [Field; 1];

    const DIM: VecIndex = 1;

    fn from_arr(arr: &Self::Arr) -> Self {
        Self(*arr)
    }
    fn from_iter(mut iter: Iter<Field>) -> Self {
        Vec1::new(*iter.next().expect(FROM_ITER_ERROR_MESSAGE))
    }
    fn get_arr(&self) -> &[Field; 1] {
        &self.0
    }
    fn iter(&self) -> std::slice::Iter<Field> {
        self.get_arr().iter()
    }
    fn map<F: Fn(Field) -> Field>(self, f: F) -> Self {
        Vec1::new(f(self[0]))
    }
    fn zip_map<F: Fn(Field, Field) -> Field>(self, rhs: Self, f: F) -> Self {
        Vec1::new(f(self[0], rhs[0]))
    }
    fn fold<F: Fn(Field, Field) -> Field>(self, init: Option<Field>, f: F) -> Field {
        let val0 = match init {
            Some(ival) => f(ival, self[0]),
            None => self[0],
        };
        f(val0, self[1])
    }
    fn dot(self, rhs: Vec1) -> Field {
        self[0] * rhs[0]
    }

    fn constant(a: Field) -> Vec1 {
        Vec1::new(a)
    }
    fn project(&self) -> Vec1 {
        Vec1::new(self[0])
    }
    fn unproject(v: Vec1) -> Vec1 {
        v
    } //this is identity since we don't have Vec0
    fn cross_product<I: std::iter::Iterator<Item = Self>>(mut vecs_iter: I) -> Self {
        if let Some(_v1) = vecs_iter.next() {
            panic!("1D cross product given more than 0 vec");
        }
        Self::zero()
    }
}

impl fmt::Display for Vec1 {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({})", self[0])
    }
}
