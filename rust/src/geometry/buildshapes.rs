use super::VertIndex;
use crate::vector::{VectorTrait};
use crate::vector::Vec3;
//use crate::vec2::Vec2;
use super::{Shape,Face,Edge};
use crate::vector::PI;
use crate::vector::Field;

pub fn build_cylinder(r : Field, h : Field, n : VertIndex) -> Shape<Vec3> {

	let angles = (0..n).map(|i| 2.0*PI*(i as Field)/(n as Field));

	let cap_coords : Vec<Vec3> = angles.map(|angle| Vec3::new(angle.cos(),angle.sin(),0.0)*r).collect();
	//let cap_coords= angles.map(|angle| Vec3(angle.cos(),angle.sin(),0.0)*r);
	let n_angles = (0..n).map(|i| 2.0*PI*((i as Field)+0.5)/(n as Field));
	let normals = n_angles.map(|angle| Vec3::new(angle.cos(),angle.sin(),0.0)*r);
	//build verts
	let top_verts = cap_coords.iter().map(|v| *v + Vec3::new(0.0,0.0,h/2.0));
	let bottom_verts = cap_coords.iter().map(|v| *v + Vec3::new(0.0,0.0,-h/2.0));

	let verts : Vec<Vec3> = top_verts.chain(bottom_verts).collect();

	// for (i,v) in verts.iter().enumerate() {
	// 	println!("vi {}: {}",i,v);
	// }

	//build edges
	let top_edges = (0..n).map(|i| Edge(i,(i+1)%n));
	let bottom_edges = (0..n).map(|i| Edge(i + n,(i+1)%n + n));
	let long_edges = (0..n).map(|i| Edge(i,i+ n));
	let edges : Vec<Edge> = top_edges.chain(bottom_edges).chain(long_edges).collect();

	// for (i,e) in edges.iter().enumerate() {
	// 	println!("ei {}: {}",i,e);
	// }

	//build faces
	let top_face = Face::new((0..n).collect(),Vec3::one_hot(2));
	let bottom_face = Face::new((n..2*n).collect(),Vec3::one_hot(2)*(-1.0));
	let long_faces = (0..n).zip(normals).map(|(i,normal)| Face::new(
		vec![i,i+n,2*n + i,2*n + (i+1)%n],
		normal));

	let faces : Vec<Face<Vec3>>  = vec![top_face,bottom_face]
		.into_iter()
		.chain(long_faces)
		.collect();
	//let faces : Face<Vec3> = Vec::new();
	// for (i,f) in faces.iter().enumerate() {
	// 	println!("fi {}: {}",i,f);
	// }

	return Shape::new(verts,edges,faces);

}

fn test_cylinder()
{
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