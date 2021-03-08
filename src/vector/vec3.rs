use std::ops::{Add,Sub,Neg,Mul,Div,Index,IndexMut};
use std::fmt;
use crate::vector::{VecIndex,VectorTrait,Field,Vec2};
use super::Mat3;

#[derive(Copy,Clone,Debug)]
pub struct Vec3{arr: [Field ; 3]}
impl Vec3 {
    pub fn new(v0 : Field, v1 : Field, v2 : Field) -> Vec3
    {
        Vec3{arr : [v0,v1,v2]}
    }
}
impl Index<VecIndex> for Vec3 {
        type Output = Field;

        fn index(&self, i: VecIndex) -> &Self::Output {
                match i {
                         0 | -3 => &self.get_arr()[0],
                         1 | -2 => &self.get_arr()[1],
                         2 | -1 => &self.get_arr()[2],
                        _ => panic!("Invalid index {} for Vec3", i)
                }
        }
}
impl IndexMut<VecIndex> for Vec3 {
    fn index_mut<'a>(&'a mut self, index: VecIndex) -> &'a mut Self::Output {
        match index {
                         0 | -3 => &mut self.arr[0],
                         1 | -2 => &mut self.arr[1],
                         2 | -1 => &mut self.arr[2],
                        _ => panic!("Invalid index {} for Vec3", index)
        }
    }
}
impl Add<Vec3> for Vec3 {
    type Output = Vec3;
    
    fn add(self, rhs: Self) -> Vec3 {
        self.zip_map(rhs,|a,b| a+b)
    }
}

impl Sub<Vec3> for Vec3 {
    type Output = Vec3;
    
    fn sub(self, rhs: Self) -> Vec3 {
    self.zip_map(rhs,|a,b| a-b)
    }
}
impl Neg for Vec3
{
    type Output = Vec3;
    
    fn neg(self) -> Vec3 {
        Vec3::zero() - self
    }
}

impl Mul<Field> for Vec3 {
    type Output = Vec3;
    
    fn mul(self, rhs: Field) -> Vec3 {
        //Vec3::new(self[0]*rhs,self[1]*rhs,self[2]*rhs)
        self.map(|v_i| v_i*rhs)
    }
}

impl Div<Field> for Vec3 {
    type Output = Vec3;
    
    fn div(self, rhs: Field) -> Vec3 {
        self*(1.0/rhs)
    }
}


impl VectorTrait for Vec3 {
    type M = Mat3;
    type SubV = Vec2;
    type Arr = [Field ; 3];

    const DIM : VecIndex = 3;

    fn from_arr(arr : &Self::Arr) -> Self
    {
        Self{arr : *arr}
    }
    fn get_arr(&self) -> &[Field ; 3]{
        &self.arr
    }
    fn iter<'a>(&'a self) -> std::slice::Iter<'a,Field> {
        self.get_arr().iter()
    }
    fn map<F : Fn(Field) -> Field>(self, f : F) -> Self {
        Vec3::new(f(self[0]),f(self[1]),f(self[2]))
    }
    fn zip_map<F : Fn(Field, Field) -> Field>(self, rhs : Self, f : F) -> Self {
        Vec3::new(f(self[0],rhs[0]),f(self[1],rhs[1]),f(self[2],rhs[2]))
    }
    fn fold<F : Fn(Field, Field) -> Field>(self, init : Option<Field>, f : F) -> Field {
        let val0 = match init {
            Some(ival) => f(ival,self[0]),
            None => self[0],
        };
        f(f(val0,self[1]),self[2])
    }
    fn dot(self, rhs: Vec3) -> Field {
        self[0]*rhs[0] + self[1]*rhs[1] + self[2]*rhs[2]
    }
    fn constant(a : Field) -> Vec3{
        Vec3::new(a,a,a)
    }
    fn project(&self) -> Vec2 {
        Vec2::new(self[0],self[1])
    }
    fn unproject(v: Self::SubV) -> Self { Self::new(v[0],v[1],0.0)}
    fn cross_product<I : std::iter::Iterator<Item=Self>>(mut vecs_iter : I) -> Self {
        let a = vecs_iter.next().expect("No vecs given to 3d cross product");
        let b = vecs_iter.next().expect("1 vec given to 3d cross product");
        if let Some(_) = vecs_iter.next() {
            panic!("3D cross product given more than 2 vecs");
        }
        Vec3::new(
             a[1]*b[2] - a[2]*b[1],
            -a[0]*b[2] + a[2]*b[0],
             a[0]*b[1] - a[1]*b[0])
    }
}

impl fmt::Display for Vec3 {
        // This trait requires `fmt` with this exact signature.
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "({},{},{})", self[0],self[1],self[2])
        }
}