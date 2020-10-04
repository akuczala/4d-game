use std::ops::{Add,Sub,Neg,Mul,Div,Index,IndexMut};
use std::fmt;
use crate::vector::{VecIndex,VectorTrait,Field};
use super::Mat2;

#[derive(Copy,Clone,Debug)]
pub struct Vec2{pub arr : [Field ; 2]}
impl Vec2 {
    pub fn new(v0 : Field, v1 : Field) -> Vec2
    {
        Vec2{arr : [v0,v1]}
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
                        _ => panic!("Invalid index {} for Vec2", i)
                }
        }
}
impl IndexMut<VecIndex> for Vec2 {
    fn index_mut<'a>(&'a mut self, index: VecIndex) -> &'a mut Self::Output {
        match index {
            0 | -2 => &mut self.arr[0],
         1 | -1=> &mut self.arr[1],
        _ => panic!("Invalid index {} for Vec2", index)
        }
    }
}


impl Add<Vec2> for Vec2 {
    type Output = Vec2;
    
    fn add(self, rhs: Self) -> Vec2 {
        self.zip_map(rhs,|a,b| a+b)
    }
}
impl Sub<Vec2> for Vec2 {
    type Output = Vec2;
    
    fn sub(self, rhs: Self) -> Vec2 {
        self.zip_map(rhs,|a,b| a-b)
    }
}
impl Neg for Vec2
{
    type Output = Vec2;
    
    fn neg(self) -> Vec2 {
        Vec2::zero() - self
    }
}

impl Mul<Field> for Vec2 {
    type Output = Vec2;
    
    fn mul(self, rhs: Field) -> Vec2 {
        //Vec2::new(self[0]*rhs,self[1]*rhs)
        self.map(|v_i| v_i*rhs)
    }
}
impl Div<Field> for Vec2 {
    type Output = Vec2;
    
    fn div(self, rhs: Field) -> Vec2 {
        self*(1.0/rhs)
    }
}
impl VectorTrait for Vec2 {
    type M = Mat2;

    //this should be Field but we have to implement VectorTrait
    //for Field
    type SubV = Vec2; 

    type Arr = [Field ; 2];

    const DIM : VecIndex = 2;
    
    fn from_arr(arr : &Self::Arr) -> Self
    {
        Self{arr : *arr}
    }
    fn get_arr(&self) -> &[Field ; 2]{
        &self.arr
    }
    fn iter<'a>(&'a self) -> std::slice::Iter<'a,Field> {
        self.get_arr().iter()
    }
    fn map<F : Fn(Field) -> Field>(self, f : F) -> Self {
        Vec2::new(f(self[0]),f(self[1]))
    }
    fn zip_map<F : Fn(Field, Field) -> Field>(self, rhs : Self, f : F) -> Self {
        Vec2::new(f(self[0],rhs[0]),f(self[1],rhs[1]))
    }
    fn fold<F : Fn(Field, Field) -> Field>(self, init : Option<Field>, f : F) -> Field {
        let val0 = match init {
            Some(ival) => f(ival,self[0]),
            None => self[0],
        };
        f(val0,self[1])
    }
    fn dot(self, rhs: Vec2) -> Field {
        self[0]*rhs[0] + self[1]*rhs[1]
    }

    fn constant(a : Field) -> Vec2 {
        Vec2::new(a,a)
    }
    //should really return Field
    //I instead just throw away the second component
    fn project(&self) -> Vec2 {
        Vec2::new(self[0],0.0)
    }
    fn cross<I : std::iter::Iterator<Item=Self>>(mut vecs_iter : I) -> Self {
        let v0 = vecs_iter.next().expect("No vecs given to 2d cross product");
        if let Some(v1) = vecs_iter.next() {
            panic!("2D cross product given more than 1 vec");
        }
        Vec2::new(-v0[1],v0[0]) //right handed
    }
}

impl fmt::Display for Vec2 {
        // This trait requires `fmt` with this exact signature.
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "({},{})", self[0],self[1])
        }
}
