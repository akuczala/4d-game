use std::ops::{Add,Sub,Neg,Mul,Div,Index,IndexMut};
use std::fmt;
use crate::vector::{VecIndex,VectorTrait,Field,Vec3,rig::Rig};
use super::Mat4;

#[derive(Copy,Clone)]
pub struct Vec4{arr: [Field ; 4]}
impl Vec4 {
  pub fn new(v0 : Field, v1 : Field, v2 : Field, v3: Field) -> Vec4
  {
    Vec4{arr : [v0,v1,v2,v3]}
  }
}
impl Index<VecIndex> for Vec4 {
    type Output = Field;

    fn index(&self, i: VecIndex) -> &Self::Output {
        match i {
             0 | -4 => &self.get_arr()[0],
             1 | -3 => &self.get_arr()[1],
             2 | -2 => &self.get_arr()[2],
             3 | -1 => &self.get_arr()[3],
            _ => panic!("Invalid index {} for Vec4", i)
        }
    }
}
impl IndexMut<VecIndex> for Vec4 {
  fn index_mut<'a>(&'a mut self, index: VecIndex) -> &'a mut Self::Output {
    match index {
             0 | -4 => &mut self.arr[0],
             1 | -3 => &mut self.arr[1],
             2 | -2 => &mut self.arr[2],
             3 | -1 => &mut self.arr[3], 
            _ => panic!("Invalid index {} for Vec3", index)
    }
  }
}
impl Add<Vec4> for Vec4 {
  type Output = Vec4;
  
  fn add(self, rhs: Self) -> Vec4 {
    self.zip_map(rhs,|a,b| a+b)
  }
}

impl Sub<Vec4> for Vec4 {
  type Output = Vec4;
  
  fn sub(self, rhs: Self) -> Vec4 {
  self.zip_map(rhs,|a,b| a-b)
  }
}
impl Neg for Vec4
{
  type Output = Vec4;
  
  fn neg(self) -> Vec4 {
    Vec4::zero() - self
  }
}

impl Mul<Field> for Vec4 {
  type Output = Vec4;
  
  fn mul(self, rhs: Field) -> Vec4 {
    self.map(|v_i| v_i*rhs)
  }
}

impl Div<Field> for Vec4 {
  type Output = Vec4;
  
  fn div(self, rhs: Field) -> Vec4 {
    self*(1.0/rhs)
  }
}


impl Rig<Field> for Vec4 {
  
  type Arr = [Field ; 4];
  type SubR = Vec3;

  const DIM : VecIndex = 4;

  fn from_arr(arr : &Self::Arr) -> Self
  {
    Self{arr : *arr}
  }
  fn get_arr(&self) -> &[Field ; 4]{
    &self.arr
  }
  fn iter<'a>(&'a self) -> std::slice::Iter<'a,Field> {
    self.get_arr().iter()
  }
  fn map<F : Fn(Field) -> Field>(self, f : F) -> Self {
    Vec4::new(f(self[0]),f(self[1]),f(self[2]),f(self[3]))
  }
  fn zip_map<F : Fn(Field, Field) -> Field>(self, rhs : Self, f : F) -> Self {
    Vec4::new(f(self[0],rhs[0]),f(self[1],rhs[1]),f(self[2],rhs[2]),f(self[3],rhs[3]))
  }
  fn fold<F : Fn(Field, Field) -> Field>(self, init : Option<Field>, f : F) -> Field {
    let val0 = match init {
      Some(ival) => f(ival,self[0]),
      None => self[0],
    };
    f(f(f(val0,self[1]),self[2]),self[3])
  }
  fn dot(self, rhs: Vec4) -> Field {
    self[0]*rhs[0] + self[1]*rhs[1] + self[2]*rhs[2] + self[3]*rhs[3]
  }
  fn constant(a : Field) -> Vec4{
    Vec4::new(a,a,a,a)
  }
  fn project(&self) -> Self::SubR {
    Self::SubR::new(self[0],self[1],self[2])
  }
}
impl VectorTrait for Vec4 {
  type M = Mat4;
  type SubV = Vec3;

  // fn project(&self) -> Self::SubV {
  //   Self::SubV::new(self[0],self[1],self[2])
  // }
}

impl fmt::Display for Vec4 {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({},{},{},{})", self[0],self[1],self[2],self[3])
    }
}