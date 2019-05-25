#[allow(dead_code)]
mod vector;
//#[allow(dead_code)]
mod geometry;
mod colors;
mod vec2; mod vec3;
mod mat2; mod mat3;
mod buildshapes;
// use vector as vect;

// use geometry as geo;
// use colors::*;

//use std::io;
use buildshapes::build_cylinder;

fn main() {
  // let line = geo::Line(vec3::Vec3(0.0,0.0,0.0),vec3::Vec3(1.0,2.0,3.0));
  // let n = vec3::Vec3(5.0,5.0,5.0);
  // let plane = geo::Plane{normal : n, threshold : -0.5};
  // let intersect = geo::line_plane_intersect(line,plane);
  // match intersect {
  //   Some(v) => println!("{}",v),
  //   None => println!("None")
  // }
  // let mut edgeis = Vec::new();
  // edgeis.push(5);
  // edgeis.push(6);
  // edgeis.push(7);
  // edgeis.push(8);
  // let face = geo::Face::new(edgeis,n,WHITE);

  let mut cylinder = build_cylinder(1.0,1.0,4);
  for face in cylinder.faces.iter() {
    println!("{}",face.center);
  }
  println!("rotate");
  cylinder.rotate(1,2,3.141592653/2.0);

  for face in cylinder.faces.iter() {
    println!("{}",face.center);
  }
}
