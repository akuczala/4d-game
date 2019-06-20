use itertools::Itertools;
use crate::vector::{VectorTrait,VecIndex};
use crate::vector::{Vec2,Vec3,Vec4,barycenter};
//use crate::vec2::Vec2;
use super::{Shape,Face,Edge,EdgeIndex,VertIndex};
use crate::vector::PI;
use crate::vector::Field;

pub fn build_prism_3d(r : Field, h : Field, n : VertIndex) -> Shape<Vec3> {

	//starting angle causes first edge to be parallel to y axis
	//lets us define a cube as a cylinder
	let angles = (0..n).map(|i| 2.0*PI*((i as Field)-0.5)/(n as Field));
	let cap_coords : Vec<Vec3> = angles.map(|angle| Vec3::new(angle.cos(),angle.sin(),0.0)*r).collect();
	
	let n_angles = (0..n).map(|i| 2.0*PI*(i as Field)/(n as Field));
	let normals = n_angles.map(|angle| Vec3::new(angle.cos(),angle.sin(),0.0)*r);

	//build verts
	let top_verts = cap_coords.iter().map(|v| *v + Vec3::new(0.0,0.0,h/2.0));
	let bottom_verts = cap_coords.iter().map(|v| *v + Vec3::new(0.0,0.0,-h/2.0));

	let verts : Vec<Vec3> = top_verts.chain(bottom_verts).collect();

	//build edges
	let top_edges = (0..n).map(|i| Edge(i,(i+1)%n));
	let bottom_edges = (0..n).map(|i| Edge(i + n,(i+1)%n + n));
	let long_edges = (0..n).map(|i| Edge(i,i+ n));
	let edges : Vec<Edge> = top_edges.chain(bottom_edges).chain(long_edges).collect();

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

	return Shape::new(verts,edges,faces);

}
pub fn build_cube_3d(length : Field) -> Shape<Vec3> {
	build_prism_3d(length/(2.0 as Field).sqrt(),length,4)
}

use itertools::multizip;

//builds 4d duoprism
//n_circ points is a length two list of # points around each perp circle
//rs is a list of radii of each circle
//each face is a prism. if circle 0 has m points and circle 1 has n points,
//there are m n-prisms and n m-prisms
pub fn build_duoprism_4d(
	radii : [Field ; 2],
	axes: [[VecIndex ; 2] ; 2],
	ns : [VertIndex ; 2]
	) -> Shape<Vec4>
{
	if axes[0] == axes[1] {
		panic!("Axes of duoprism must be distinct")
	}
	let ns_copy = ns.clone();
	let angles = ns_copy.iter().map(move |n| (0..*n)
		.map(move |i| 2.0*PI*((i as Field)-0.5)/(*n as Field)));
	let circle_coords : Vec<Vec<Vec2>> = multizip((radii.iter(),angles)).
	map(|(r,angles)| angles.map(|angle| Vec2::new(angle.cos(),angle.sin())*(*r)).collect()
		).collect();

	let verts : Vec<Vec4> = iproduct!(circle_coords[0].iter(),circle_coords[1].iter())
		.map(|(c0,c1)| {
			let mut v = Vec4::zero();
			v[axes[0][0]] = c0[0];
		    v[axes[0][1]] = c0[1];
		    v[axes[1][0]] = c1[0];
		    v[axes[1][1]] = c1[1];
		    v
		})
		.collect();
	for v in &verts {
		println!("{}",v)
	}

	//we need m loops of length n and n loops of length m
	let edges_1 = iproduct!((0..ns[0]),0..ns[1])
		.map(|(i,j)| Edge(
			j+i*ns[1],
			(j+1)%ns[1]+i*ns[1]
			));
	let edges_2 = iproduct!((0..ns[0]),0..ns[1])
		.map(|(i,j)| Edge(
			j+i*ns[1],
			j+((i+1)%ns[0])*ns[1])
		);
	let edges : Vec<Edge>= edges_1.chain(edges_2).collect();
	fn make_normal(
		edgeis : &Vec<EdgeIndex>,
		verts : &Vec<Vec4>,
		edges : &Vec<Edge>
		) -> Vec4 {
		let vertis : Vec<VertIndex> = edgeis.iter()
			.map(|ei| &edges[*ei])
			.map(|edge| vec![edge.0,edge.1])
			.flat_map(|x| x)
			.collect(); //would like to not have to collect here
		//get unique values
		let vertis : Vec<VertIndex> = vertis.into_iter().unique().collect();
		let verts_in_face : Vec<Vec4> = vertis.iter()
			.map(|vi| verts[*vi])
			.collect();
		let center = barycenter(verts_in_face);
		center.normalize()
	}
	// we need m n-prisms and n m-prisms
	fn make_face1(
		i : VertIndex,
		ns : &[VertIndex ; 2],
		verts : &Vec<Vec4>,
		edges : &Vec<Edge>) -> Face<Vec4> {
		let (m,n) = (ns[0],ns[1]);
		let cap1_edgeis = (0..n).map(|j| j + i*n);
		let cap2_edgeis = (0..n).map(|j| j + ((i+1)%m)*n);
		let long_edgeis = (0..n).map(|j| m*n + j + i*n);
		let edgeis : Vec<EdgeIndex> = cap1_edgeis
			.chain(cap2_edgeis).chain(long_edgeis).collect();
		let normal = make_normal(&edgeis,&verts,&edges);
		Face::new(edgeis,normal)
	}
	fn make_face2(
		j : VertIndex,
		ns : &[VertIndex ; 2],
		verts : &Vec<Vec4>,
		edges : &Vec<Edge>) -> Face<Vec4> {
		let (m,n) = (ns[0],ns[1]);
		let cap1_edgeis = (0..m).map(|i| m*n + j + i*n);
		let cap2_edgeis = (0..m).map(|i| m*n + (j+1)%n + i*n);
		let long_edgeis = (0..m).map(|i| j + i*n);
		let edgeis : Vec<EdgeIndex> = cap1_edgeis
			.chain(cap2_edgeis).chain(long_edgeis).collect();
		let normal = make_normal(&edgeis,&verts,&edges);
		Face::new(edgeis,normal)
	}
	let faces_1 = (0..ns[0]).map(|i| make_face1(i,&ns.clone(),&verts,&edges));
	let faces_2 = (0..ns[1]).map(|j| make_face2(j,&ns.clone(),&verts,&edges));
	let faces : Vec<Face<Vec4>> = faces_1.chain(faces_2).collect();
	Shape::new(verts,edges,faces)


}