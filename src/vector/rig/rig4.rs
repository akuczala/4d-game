use crate::vector::rig::RigField;
use crate::vector::rig::Rig;
use std::ops::{Add,Sub,Neg,Mul,Div,Index,IndexMut};
use std::fmt;
use crate::vector::{VecIndex,VectorTrait,Field,Vec3};

#[derive(Copy,Clone)]
pub struct Rig4<T : RigField>{arr: [T ; 4]}
impl<T : RigField> Rig4<T> {
  pub fn new(v0 : T, v1 : T, v2 : T, v3: T) -> Rig4<T>
  {
    Rig4{arr : [v0,v1,v2,v3]}
  }
}
impl<T : RigField> Index<VecIndex> for Rig4<T> {
    type Output = T;

    fn index(&self, i: VecIndex) -> &Self::Output {
        match i {
             0 | -4 => &self.get_arr()[0],
             1 | -3 => &self.get_arr()[1],
             2 | -2 => &self.get_arr()[2],
             3 | -1 => &self.get_arr()[3],
            _ => panic!("Invalid index {} for Rig4", i)
        }
    }
}
impl<T : RigField> IndexMut<VecIndex> for Rig4<T> {
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
impl<T : RigField> Add<Rig4<T>> for Rig4<T> {
  type Output = Rig4<T>;
  
  fn add(self, rhs: Self) -> Rig4<T> {
    self.zip_map(rhs,|a,b| a+b)
  }
}

impl<T : RigField> Mul<T> for Rig4<T> {
  type Output = Rig4<T>;
  
  fn mul(self, rhs: T) -> Rig4<T> {
    self.map(|v_i| v_i*rhs)
  }
}

impl<T : RigField> Rig<T> for Rig4<T> {

  type Arr = [T ; 4];
  type SubR = Rig4<T>; //should be Rig3 here but its not implemented yet

  const DIM : VecIndex = 4;

  fn from_arr(arr : &Self::Arr) -> Self
  {
    Self{arr : *arr}
  }
  fn get_arr(&self) -> &[T ; 4]{
    &self.arr
  }
  fn iter<'a>(&'a self) -> std::slice::Iter<'a,T> {
    self.get_arr().iter()
  }
  fn map<F : Fn(T) -> T>(self, f : F) -> Self {
    Rig4::new(f(self[0]),f(self[1]),f(self[2]),f(self[3]))
  }
  fn zip_map<F : Fn(T, T) -> T>(self, rhs : Self, f : F) -> Self {
    Rig4::new(f(self[0],rhs[0]),f(self[1],rhs[1]),f(self[2],rhs[2]),f(self[3],rhs[3]))
  }
  fn fold<F : Fn(T, T) -> T>(self, init : Option<T>, f : F) -> T {
    let val0 = match init {
      Some(ival) => f(ival,self[0]),
      None => self[0],
    };
    f(f(f(val0,self[1]),self[2]),self[3])
  }
  fn dot(self, rhs: Rig4<T>) -> T {
    self[0]*rhs[0] + self[1]*rhs[1] + self[2]*rhs[2] + self[3]*rhs[3]
  }
  fn constant(a : T) -> Rig4<T> {
    Rig4::new(a,a,a,a)
  }
  fn project(&self) -> Self::SubR {
    Self::SubR::new(self[0],self[1],self[2],self[3]) //should be Rig3 but not implemented yet
  }
}

impl<T : RigField> fmt::Display for Rig4<T> {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({},{},{},{})", self[0],self[1],self[2],self[3])
    }
}