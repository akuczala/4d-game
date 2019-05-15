use fmt::Display;
use std::ops::{Add,Sub,Mul,Div};
use std::fmt;
//use alga::linear::FiniteDimInnerSpace;
pub type Field = f64;
const EPSILON : Field = 0.0001;

#[derive(Copy,Clone)]
pub struct Vec2(pub Field,pub Field);
#[derive(Copy,Clone)]
pub struct Vec3(pub Field,pub Field,pub Field);

pub trait VectorTrait: Copy + Display +
 Add<Output=Self> + Sub<Output=Self> + Mul<Field,Output=Self> + Div<Field,Output=Self> {

  type M : MatrixTrait<Self>;
  fn dot(self, rhs: Self) -> Field;
  fn norm_sq(self) -> Field {
    self.dot(self)
  }
  fn norm(self) -> Field {
    self.norm_sq().sqrt()
  }
  fn normalize(self) -> Self {
    self/self.norm()
  }
  fn is_close(v1: Self, v2: Self) -> bool {
    (v1-v2).norm_sq() < EPSILON*EPSILON
  }
  fn zero() -> Self;
  fn ones() -> Self;
  fn linterp(v1: Self, v2: Self,x : Field) -> Self {
    v1*(1.-x) + v2*x
  }
}

//impl<T> Foo for T where T: Clone + Mul<i64> + Add<i64> + ... {}

//fn foo<C>() where i64: From<C>, C: Foo {}

impl Add<Vec2> for Vec2 {
  type Output = Vec2;
  
  fn add(self, rhs: Self) -> Vec2 {
    Vec2(self.0+rhs.0,self.1+rhs.1)
  }
}

impl Add<Vec3> for Vec3 {
  type Output = Vec3;
  
  fn add(self, rhs: Self) -> Vec3 {
  Vec3(self.0+rhs.0,self.1+rhs.1,self.2+rhs.2)
  }
}

impl Sub<Vec2> for Vec2 {
  type Output = Vec2;
  
  fn sub(self, rhs: Self) -> Vec2 {
    Vec2(self.0-rhs.0,self.1-rhs.1)
  }
}

impl Sub<Vec3> for Vec3 {
  type Output = Vec3;
  
  fn sub(self, rhs: Self) -> Vec3 {
  Vec3(self.0-rhs.0,self.1-rhs.1,self.2-rhs.2)
  }
}

impl Mul<Field> for Vec2 {
  type Output = Vec2;
  
  fn mul(self, rhs: Field) -> Vec2 {
    Vec2(self.0*rhs,self.1*rhs)
  }
}

impl Mul<Field> for Vec3 {
  type Output = Vec3;
  
  fn mul(self, rhs: Field) -> Vec3 {
    Vec3(self.0*rhs,self.1*rhs,self.2*rhs)
  }
}

impl Div<Field> for Vec2 {
  type Output = Vec2;
  
  fn div(self, rhs: Field) -> Vec2 {
    Vec2(self.0/rhs,self.1/rhs)
  }
}

impl Div<Field> for Vec3 {
  type Output = Vec3;
  
  fn div(self, rhs: Field) -> Vec3 {
    Vec3(self.0/rhs,self.1/rhs,self.2/rhs)
  }
}

impl VectorTrait for Vec2 {
  type M = Mat2;
  fn norm(self) -> Field {
    (self.0*self.0 + self.1*self.1).sqrt()
  }
  fn dot(self, rhs: Vec2) -> Field {
    (self.0*rhs.0 + self.1*rhs.1)
  }
  fn zero() -> Vec2{
    Vec2(0.,0.)
  }
  fn ones() -> Vec2{
    Vec2(1.,1.)
  }
}
impl VectorTrait for Vec3 {
  type M = Mat3;
  fn norm(self) -> Field {
    (self.0*self.0 + self.1*self.1 + self.2*self.2).sqrt()
  }
  fn dot(self, rhs: Vec3) -> Field {
    (self.0*rhs.0 + self.1*rhs.1 + self.2*rhs.2)
  }
  fn zero() -> Vec3{
    Vec3(0.,0.,0.)
  }
  fn ones() -> Vec3{
    Vec3(1.,1.,1.)
  }
}

impl fmt::Display for Vec2 {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{},{}", self.0,self.1)
    }
}
impl fmt::Display for Vec3 {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{},{},{}", self.0,self.1,self.2)
    }
}

pub trait MatrixTrait<V>: Display + Copy + Add<Output=Self> + Sub<Output=Self> 
//+ Mul<Field,Output=Self> + Mul<Self,Output=Self>
 + Mul<V,Output=V> //weirdly only the last Mul is remembered
{
  fn transpose(self) -> Self;
  fn outer(v1: V, v2: V) -> Self;
  fn id() -> Self;
  fn dot(self,rhs: Self) -> Self; //matrix multiplication
}

#[derive(Copy,Clone)]
pub struct Mat3(pub Vec3,pub Vec3,pub Vec3);

impl Mat3{
  pub fn new(c : &[Field; 9]) -> Mat3 {
    Mat3(Vec3(c[0],c[1],c[2]),Vec3(c[3],c[4],c[5]),Vec3(c[6],c[7],c[8]))
  }
}
impl Add<Mat3> for Mat3 {
  type Output = Mat3;
  
  fn add(self, rhs: Self) -> Mat3 {
  Mat3(self.0+rhs.0,self.1+rhs.1,self.2+rhs.2)
  }
}

impl Sub<Mat3> for Mat3 {
  type Output = Mat3;
  
  fn sub(self, rhs: Self) -> Mat3 {
  Mat3(self.0-rhs.0,self.1-rhs.1,self.2-rhs.2)
  }
}

impl Mul<Field> for Mat3 {
  type Output = Mat3;
  
  fn mul(self, rhs: Field) -> Mat3 {
    Mat3(self.0*rhs,self.1*rhs,self.2*rhs)
  }
}

impl Mul<Vec3> for Mat3 {
  type Output = Vec3;
  
  fn mul(self, rhs: Vec3) -> Vec3 {
    Vec3(self.0.dot(rhs), self.1.dot(rhs), self.2.dot(rhs))
  }
}
impl MatrixTrait<Vec3> for Mat3 {
//  type V = Vec3;

  fn transpose(self) -> Mat3 {
    Mat3(Vec3((self.0).0,(self.1).0,(self.2).0),
    Vec3((self.0).1,(self.1).1,(self.2).1),
    Vec3((self.0).2,(self.1).2,(self.2).2))
  }
  fn outer(v1: Vec3, v2: Vec3) -> Mat3 {
    Mat3(v2 * v1.0, v2 * v1.1, v2 * v1.2)
  }
  fn id() -> Mat3 {
    Mat3::new(&
      [1.,0.,0.,
      0.,1.,0.,
      0.,0.,1.])
  }
  fn dot(self, rhs: Mat3) -> Mat3 {
    let rhs_t = rhs.transpose();
    Mat3(self * (rhs_t).0, self * (rhs_t).1, self * (rhs_t).2).transpose()
  }
}

#[derive(Copy,Clone)]
pub struct Mat2(pub Vec2,pub Vec2);

impl Mat2{
  pub fn new(c : &[Field; 4]) -> Mat2 {
    Mat2(Vec2(c[0],c[1]),Vec2(c[2],c[3]))
  }
}
impl Add<Mat2> for Mat2 {
  type Output = Mat2;
  
  fn add(self, rhs: Self) -> Mat2 {
  Mat2(self.0+rhs.0,self.1+rhs.1)
  }
}

impl Sub<Mat2> for Mat2 {
  type Output = Mat2;
  
  fn sub(self, rhs: Self) -> Mat2 {
  Mat2(self.0-rhs.0,self.1-rhs.1)
  }
}

impl Mul<Field> for Mat2 {
  type Output = Mat2;
  
  fn mul(self, rhs: Field) -> Mat2 {
    Mat2(self.0*rhs,self.1*rhs)
  }
}

impl Mul<Vec2> for Mat2 {
  type Output = Vec2;
  
  fn mul(self, rhs: Vec2) -> Vec2 {
    Vec2(self.0.dot(rhs), self.1.dot(rhs))
  }
}

impl MatrixTrait<Vec2> for Mat2 {
 // type V = Vec2;

  fn transpose(self) -> Mat2 {
    Mat2(Vec2((self.0).0,(self.1).0),
    Vec2((self.0).1,(self.1).1))
  }
  fn outer(v1: Vec2, v2: Vec2) -> Mat2 {
    Mat2(v2 * v1.0, v2 * v1.1)
  }
  fn id() -> Mat2 {
    Mat2::new(&
      [1.,0.,
      0.,1.])
  }
  //probably not correct
  fn dot(self, rhs: Mat2) -> Mat2 {
    let rhs_t = rhs.transpose();
    Mat2(self * (rhs_t).0, self * (rhs_t).1).transpose()
  }
}

impl fmt::Display for Mat2 {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} \n {}", self.0,self.1)
    }
}
impl fmt::Display for Mat3 {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} \n {} \n {}", self.0,self.1,self.2)
    }
}