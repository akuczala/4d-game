mod vector;
use vector::{VectorTrait,MatrixTrait,Field,Vec2,Vec3,Mat2,Mat3};



fn rotation_matrix<V>(v1 : V, v2: V, th : Option<Field>)-> V::M
where V : VectorTrait
{
  let u = v1/v1.norm();
  let v = v2/v2.norm();
   let costh = match th {
   None => u.dot(v),
   Some(angle) => angle.cos()
 };
 let sinth = match th {
   None => (1.0 - (costh*costh).min(1.0)).sqrt(),
   Some(angle) => angle.sin()
 };

 let w = v - u * u.dot(v);
 let w = w.normalize();

 let r1 = u*costh - w*sinth;
 let r2 = u*sinth + w*costh;

 V::M::id() + V::M::outer(u,r1-u) + V::M::outer(w,r2-w)
}
fn diagnostic<V>(v1: V, v2: V)
where V: VectorTrait
{
  let mat = rotation_matrix(v1,v2,None);
  println!("rot mat:{}",mat);
  println!("{}",mat.dot(mat.transpose()));
  println!("correct rotation: {}",V::is_close(v2.normalize(),mat * v1.normalize()));
}

fn main() {
    let v1 = Vec2(1.0,2.0);
    let v2 = Vec2(3.0,4.0);
    diagnostic(v1,v2);

    let v1 = Vec3(1.0,2.0,3.0);
    let v2 = Vec3(4.0,5.0,6.0);
    diagnostic(v1,v2);

    println!("Test matrix mult 2d");
    let mat1 = Mat2::new(&[1.0,2.0,3.0,4.0]);
    let mat2 = Mat2::new(&[5.0,6.0,7.0,8.0]);
    println!("{}",mat1.dot(mat2));
    println!("Test matrix mult 3d");
    let mat1 = Mat3::new(&[1.0,2.0,3.0,4.0,5.0,6.0,7.0,8.0,9.0]);
    let mat2 = Mat3::new(&[10.,11.,12.,13.,14.,15.,16.,17.,18.]);
    println!("{}",mat1.dot(mat2));
  }
