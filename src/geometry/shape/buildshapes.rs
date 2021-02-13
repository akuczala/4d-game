use itertools::Itertools;
use crate::vector::{VectorTrait,MatrixTrait,VecIndex};
use crate::vector::{Vec2,Vec3,Vec4,barycenter};
//use crate::vec2::Vec2;
use super::{Shape,Face,Edge,EdgeIndex,VertIndex,FaceIndex};
use crate::vector::PI;
use crate::vector::Field;
use crate::colors::*;
use crate::draw::Texture;
use std::marker::PhantomData;

pub struct ShapeBuilder<V : VectorTrait>(PhantomData<V>);
impl ShapeBuilder<Vec2> {
	pub fn build_cube(length : Field) -> Shape<Vec2> {
		build_prism_2d(length/(2.0 as Field).sqrt(),4)
	}
}
impl ShapeBuilder<Vec3> {
	pub fn build_cube(length : Field) -> Shape<Vec3> {
		build_cube_3d(length)
	}
	pub fn build_coin() -> Shape<Vec3> {
		build_prism_3d(0.1,0.025,10)
		//build_cube_3d(0.2)
            .set_color(YELLOW)
	}
}
impl ShapeBuilder<Vec4> {
	pub fn build_cube(length : Field) -> Shape<Vec4> {
		build_cube_4d(length)
	}
	pub fn build_coin() -> Shape<Vec4> {
		build_duoprism_4d([0.1,0.025],[[0,1],[2,3]],[10,4])
            .set_color(YELLOW)
	}
}

pub fn build_test_face() -> Shape<Vec3> {
	type v = Vec3;
	let basis = <v as VectorTrait>::M::id();
	let shape = Shape::new(
		vec![v::zero(),basis[0],basis[0] + basis[1], basis[1]],
		vec![Edge(0,1),Edge(1,2),Edge(2,3),Edge(3,0)],
		vec![Face::new(vec![0,1,2,3], basis[-1])]
	);
	shape
}

pub fn build_prism_2d(r : Field, n : VertIndex) -> Shape<Vec2> {

	//starting angle causes first edge to be parallel to y axis
	//lets us define a cube as a cylinder
	let angles = (0..n).map(|i| 2.0*PI*((i as Field)-0.5)/(n as Field));
	let verts : Vec<Vec2> = angles.map(|angle| Vec2::new(angle.cos(),angle.sin())*r).collect();
	
	let n_angles = (0..n).map(|i| 2.0*PI*(i as Field)/(n as Field));
	let normals = n_angles.map(|angle| Vec2::new(angle.cos(),angle.sin()));

	//build edges
	let edges = (0..n).map(|i| Edge(i,(i+1)%n)).collect();
	
	//build faces
	let faces = (0..n).zip(normals)
		.map(|(i,normal)| Face::new(vec![i], normal))
		.collect();

	return Shape::new(verts,edges,faces);

}

pub fn build_prism_3d(r : Field, h : Field, n : VertIndex) -> Shape<Vec3> {

	//starting angle causes first edge to be parallel to y axis
	//lets us define a cube as a cylinder
	let angles = (0..n).map(|i| 2.0*PI*((i as Field)-0.5)/(n as Field));
	let cap_coords : Vec<Vec3> = angles.map(|angle| Vec3::new(angle.cos(),angle.sin(),0.0)*r).collect();
	
	let n_angles = (0..n).map(|i| 2.0*PI*(i as Field)/(n as Field));
	let normals = n_angles.map(|angle| Vec3::new(angle.cos(),angle.sin(),0.0));

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
pub fn build_long_cube_3d(length : Field, width: Field) -> Shape<Vec3> {
	build_prism_3d(width/(2.0 as Field).sqrt(),length,4)
}
pub fn build_tube_cube_3d(length : Field, width: Field) -> Shape<Vec3> {
	let rect = build_prism_3d(width/(2.0 as Field).sqrt(),length,4);
	remove_faces(rect,vec![0,1])
	
}

pub fn remove_face<V : VectorTrait>(shape : Shape<V>, face_index : FaceIndex) -> Shape<V> {
	let verts = shape.verts_ref; let edges = shape.edges; let mut faces = shape.faces;
	faces.remove(face_index);
	Shape::new(verts,edges,faces)
}
pub fn remove_faces<V : VectorTrait>(shape : Shape<V>, faceis : Vec<FaceIndex>) -> Shape<V> {
	let verts = shape.verts_ref; let edges = shape.edges; let faces = shape.faces;
	let new_faces = faces.into_iter().enumerate()
		.filter(|(i,_face)| !faceis.contains(i))
		.map(|(_i,face)| face)
		.collect();
	Shape::new(verts,edges,new_faces)
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
	// for v in &verts {
	// 	println!("{}",v)
	// }

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

	// for edge in &edges {
	// 	println!("{}",edge)
	// }

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
		let center = barycenter(&verts_in_face);
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

	// for face in &faces {
	// 	println!("{}",face)
	// }

	Shape::new(verts,edges,faces)


}
pub fn build_cube_4d(length : Field) -> Shape<Vec4> {
	let r = length/(2.0 as Field).sqrt();
	build_duoprism_4d([r,r],
		[[0,1],[2,3]],
		[4,4])
}

pub fn color_cube< V: VectorTrait>(mut cube : Shape<V>) -> Shape<V> {
	let face_colors = vec![RED,GREEN,BLUE,CYAN,MAGENTA,YELLOW,ORANGE,WHITE];
    for (face, &color) in cube.faces.iter_mut().zip(&face_colors) {
        face.texture = Texture::DefaultLines{color : color.set_alpha(0.5)};
    }
    cube
}
pub fn build_axes_cubes_4d() -> Vec<Shape<Vec4>> {
    let mut shapes : Vec<Shape<Vec4>> = Vec::new();

    for i in (0..4).into_iter() {
        for sign in vec![-1.0,1.0] {
            let cube = build_cube_4d(1.0)
                .set_pos(&(<Vec4 as VectorTrait>::M::id()[i]*sign*3.0));
            shapes.push(color_cube(cube));
        }
    }
    shapes
}

pub fn cubeidor_3d() -> Vec<Shape<Vec3>> {
	let mut shapes : Vec<Shape<Vec3>> = Vec::new();

    for (i, sign, z) in iproduct!((0..2).into_iter(),vec![-1.0,1.0],(0..4)) {
    	let pos = <Vec3 as VectorTrait>::M::id()[i]*sign*1.0 + <Vec3 as VectorTrait>::one_hot(-1)*(z as Field);
            let cube = build_cube_3d(1.0)
                .set_pos(&pos);
            shapes.push(color_cube(cube));
    }
    shapes
}
pub fn cubeidor_4d() -> Vec<Shape<Vec4>> {
	let mut shapes : Vec<Shape<Vec4>> = Vec::new();

    for (i, sign, z) in iproduct!((0..3).into_iter(),vec![-1.0,1.0],(0..4)) {
    	let pos = <Vec4 as VectorTrait>::M::id()[i]*sign*1.0
    		+ <Vec4 as VectorTrait>::one_hot(-1)*1.0*(z as Field);
            let cube = build_cube_4d(1.0)
                .set_pos(&pos);
            shapes.push(color_cube(cube));
    }
    shapes
}

pub fn tube_test_3d() -> Vec<Shape<Vec3>> {
	//buildshapes::cubeidor_3d()
    let mut tube = build_long_cube_3d(4.0,1.0).set_pos(&Vec3::new(0.5,0.0,0.0));
    let mut tube2 = tube.clone().set_pos(&Vec3::new(3.0,0.0,2.5));

    //let mut tube = buildshapes::invert_normals(&tube);
    //let mut tube2 = buildshapes::invert_normals(&tube2);
    tube2.rotate(0,-1,PI/2.0);

    let cube = build_cube_3d(1.0).set_pos(&Vec3::new(0.5,0.0,2.5));
    //let cube = buildshapes::invert_normals(&cube);

    let tube = tube.set_color(RED);
    let tube2 = tube2.set_color(GREEN);
    vec![tube,tube2,cube]
}
pub fn test_3d() -> Vec<Shape<Vec3>> {
	let mut cube = build_cube_3d(1.0);
    let face_colors = vec![RED,GREEN,BLUE,CYAN,MAGENTA,YELLOW];
    for (face, color) in cube.faces.iter_mut().zip(face_colors) {
        face.texture = Texture::DefaultLines{color};
    }
    let cylinder = build_prism_3d(1.0,1.0,8)
        .set_pos(&Vec3::new(2.0,0.0,0.0));;

    let prism = build_prism_3d(1.0,1.0,3)
        .set_pos(&Vec3::new(0.0,0.0,3.0));
    vec![cube,cylinder,prism]
}

pub fn invert_normals<V : VectorTrait>(shape : &Shape<V>) -> Shape<V> {
	let mut new_shape = shape.clone();
	for face in &mut new_shape.faces {
		face.normal_ref = -face.normal_ref;
	}
	new_shape.update();
	new_shape
}

pub fn color_duocylinder(shape : &mut Shape<Vec4>, m : usize, n : usize) {
    for (i, face) in itertools::enumerate(shape.faces.iter_mut()) {
        let iint = i as i32;
        let color = Color([((iint%(m as i32)) as f32)/(m as f32),(i as f32)/((m+n) as f32),1.0,1.0]);
        face.texture = Texture::DefaultLines{color};
    }
}