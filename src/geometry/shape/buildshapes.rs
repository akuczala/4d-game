use super::{Edge, EdgeIndex, Face, FaceIndex, Shape, ShapeType, SingleFace, VertIndex};
use crate::draw::{Texture, TextureMapping};
use crate::geometry::Transformable;
use crate::graphics::colors::*;
use crate::vector::Field;
use crate::vector::PI;
use crate::vector::{barycenter, Vec2, Vec3, Vec4};
use crate::vector::{VecIndex, VectorTrait};
use itertools::Itertools;
use std::marker::PhantomData;

#[derive(Clone)]
pub struct ShapeBuilder<V> {
    pub shape: Shape<V>,
}
impl<V: VectorTrait> ShapeBuilder<V> {
    pub fn new(shape: Shape<V>) -> Self {
        Self { shape }
    }
    pub fn build_cube(length: Field) -> Self {
        let cube = match V::DIM {
            2 => build_prism_2d(length / (2.0 as Field).sqrt(), 4),
            3 => build_prism_3d(length / (2.0 as Field).sqrt(), length, 4),
            4 => {
                let r = length / (2.0 as Field).sqrt();
                build_duoprism_4d([r, r], [[0, 1], [2, 3]], [4, 4])
            }
            _ => panic!("build_cube not supported in {} dim", V::DIM),
        };
        Self::new(cube)
    }
    pub fn build_prism(dim: VecIndex, lengths: &[Field], n_sides: &[usize]) -> Self {
        Self::new(match dim {
            2 => build_prism_2d(lengths[0], n_sides[0]),
            3 => build_prism_3d(lengths[0], lengths[1], n_sides[0]),
            4 => build_duoprism_4d(
                [lengths[0], lengths[1]],
                [[0, 3], [1, 2]],
                [n_sides[0], n_sides[1]],
            ),
            _ => panic!("build_prism unsupported in {} dim", V::DIM),
        })
    }
    pub fn build_coin() -> Self {
        let coin = match V::DIM {
            2 => build_prism_2d(0.1, 10),
            3 => build_prism_3d(0.1, 0.025, 10),
            4 => build_duoprism_4d([0.1, 0.025], [[0, 1], [2, 3]], [10, 4]),
            _ => panic!("build_coin not supported in {} dim", V::DIM),
        };
        Self::new(coin)
    }
    pub fn stretch(mut self, scales: Scaling<V>) -> Self {
        self.shape.modify(&Transform::identity().with_scale(scales));
        self
    }
    pub fn build(self) -> Shape<V> {
        self.shape
    }
}

impl<V: VectorTrait> Transformable<V> for ShapeBuilder<V> {
    fn transform(&mut self, transformation: Transform<V, V::M>) {
        self.shape.modify(&transformation)
    }
}

pub fn convex_shape_to_face_shape<V: VectorTrait>(
    convex_shape: Shape<V::SubV>,
    two_sided: bool,
) -> Shape<V> {
    let subface_vertis: Vec<Vec<usize>> = convex_shape
        .faces
        .iter()
        .map(|face| face.vertis.clone())
        .collect();
    let verts = convex_shape
        .verts
        .iter()
        .map(|&v| V::unproject(v))
        .collect_vec();
    let edges = convex_shape.edges.clone();
    let face = Face::new(
        (0..convex_shape.edges.len()).collect_vec(),
        V::one_hot(-1),
        two_sided,
    );
    Shape::new_single_face(verts, edges, face, &subface_vertis)
}

fn circle_vec<V: VectorTrait>(angle: Field) -> V {
    V::one_hot(0) * angle.cos() + V::one_hot(1) * angle.sin()
}
pub fn build_prism_2d<V: VectorTrait>(r: Field, n: VertIndex) -> Shape<V> {
    //starting angle causes first edge to be parallel to y axis
    //lets us define a cube as a cylinder
    let angles = (0..n).map(|i| 2.0 * PI * ((i as Field) - 0.5) / (n as Field));
    let verts: Vec<V> = angles.map(|angle| circle_vec::<V>(angle) * r).collect();

    let n_angles = (0..n).map(|i| 2.0 * PI * (i as Field) / (n as Field));
    let normals = n_angles.map(|angle| circle_vec(angle));

    //build edges
    let edges = (0..n).map(|i| Edge(i, (i + 1) % n)).collect();

    //build faces
    let faces = (0..n)
        .zip(normals)
        .map(|(i, normal)| Face::new(vec![i], normal, false))
        .collect();

    Shape::new_convex(verts, edges, faces)
}

pub fn build_prism_3d<V: VectorTrait>(r: Field, h: Field, n: VertIndex) -> Shape<V> {
    if V::DIM < 3 {
        panic!("Can't embed 3d prism into {} dims", V::DIM)
    }
    //starting angle causes first edge to be parallel to y axis
    //lets us define a cube as a cylinder
    let angles = (0..n).map(|i| 2.0 * PI * ((i as Field) - 0.5) / (n as Field));
    let cap_coords: Vec<V> = angles.map(|angle| circle_vec::<V>(angle) * r).collect();

    let n_angles = (0..n).map(|i| 2.0 * PI * (i as Field) / (n as Field));
    let normals = n_angles.map(|angle| circle_vec::<V>(angle));

    //build verts
    let top_verts = cap_coords.iter().map(|v| *v + V::one_hot(2) * (h / 2.0));
    let bottom_verts = cap_coords.iter().map(|v| *v + V::one_hot(2) * (-h / 2.0));

    let verts: Vec<V> = top_verts.chain(bottom_verts).collect();

    //build edges
    let top_edges = (0..n).map(|i| Edge(i, (i + 1) % n));
    let bottom_edges = (0..n).map(|i| Edge(i + n, (i + 1) % n + n));
    let long_edges = (0..n).map(|i| Edge(i, i + n));
    let edges: Vec<Edge> = top_edges.chain(bottom_edges).chain(long_edges).collect();

    //build faces
    let top_face = Face::new((0..n).collect(), V::one_hot(2), false);
    let bottom_face = Face::new((n..2 * n).collect(), V::one_hot(2) * (-1.0), false);
    let long_faces = (0..n).zip(normals).map(|(i, normal)| {
        Face::new(
            vec![i, i + n, 2 * n + i, 2 * n + (i + 1) % n],
            normal,
            false,
        )
    });

    let faces: Vec<Face<V>> = vec![top_face, bottom_face]
        .into_iter()
        .chain(long_faces)
        .collect();

    Shape::new_convex(verts, edges, faces)
}
pub fn build_long_cube_3d<V: VectorTrait>(length: Field, width: Field) -> Shape<V> {
    build_prism_3d(width / (2.0 as Field).sqrt(), length, 4)
}
pub fn build_tube_cube_3d<V: VectorTrait>(length: Field, width: Field) -> Shape<V> {
    let rect = build_prism_3d(width / (2.0 as Field).sqrt(), length, 4);
    remove_faces(rect, vec![0, 1])
}

pub fn remove_face<V: VectorTrait>(shape: Shape<V>, face_index: FaceIndex) -> Shape<V> {
    let verts = shape.verts;
    let edges = shape.edges;
    let mut faces = shape.faces;
    faces.remove(face_index);
    Shape::new_convex(verts, edges, faces)
}
pub fn remove_faces<V: VectorTrait>(shape: Shape<V>, faceis: Vec<FaceIndex>) -> Shape<V> {
    let verts = shape.verts;
    let edges = shape.edges;
    let faces = shape.faces;
    let new_faces = faces
        .into_iter()
        .enumerate()
        .filter(|(i, _face)| !faceis.contains(i))
        .map(|(_i, face)| face)
        .collect();
    Shape::new_convex(verts, edges, new_faces)
}
use crate::geometry::transform::Scaling;
use crate::geometry::Transform;
use itertools::multizip;

//builds 4d duoprism
//n_circ points is a length two list of # points around each perp circle
//rs is a list of radii of each circle
//each face is a prism. if circle 0 has m points and circle 1 has n points,
//there are m n-prisms and n m-prisms
pub fn build_duoprism_4d<V: VectorTrait>(
    radii: [Field; 2],
    axes: [[VecIndex; 2]; 2],
    ns: [VertIndex; 2],
) -> Shape<V> {
    if V::DIM < 4 {
        panic!("Can't build duoprism in {} dimensions", { V::DIM })
    }
    if axes[0] == axes[1] {
        panic!("Axes of duoprism must be distinct")
    }
    let ns_copy = ns;
    let angles = ns_copy
        .iter()
        .map(move |n| (0..*n).map(move |i| 2.0 * PI * ((i as Field) - 0.5) / (*n as Field)));
    let circle_coords: Vec<Vec<Vec2>> = multizip((radii.iter(), angles))
        .map(|(&r, angles)| angles.map(|angle| circle_vec::<Vec2>(angle) * r).collect())
        .collect();

    let verts: Vec<V> = iproduct!(circle_coords[0].iter(), circle_coords[1].iter())
        .map(|(c0, c1)| {
            let mut v = V::zero();
            v[axes[0][0]] = c0[0];
            v[axes[0][1]] = c0[1];
            v[axes[1][0]] = c1[0];
            v[axes[1][1]] = c1[1];
            v
        })
        .collect();

    //we need m loops of length n and n loops of length m
    let edges_1 = iproduct!((0..ns[0]), 0..ns[1])
        .map(|(i, j)| Edge(j + i * ns[1], (j + 1) % ns[1] + i * ns[1]));
    let edges_2 = iproduct!((0..ns[0]), 0..ns[1])
        .map(|(i, j)| Edge(j + i * ns[1], j + ((i + 1) % ns[0]) * ns[1]));
    let edges: Vec<Edge> = edges_1.chain(edges_2).collect();

    fn make_normal<V: VectorTrait>(edgeis: &[EdgeIndex], verts: &[V], edges: &[Edge]) -> V {
        let vertis: Vec<VertIndex> = edgeis
            .iter()
            .map(|ei| &edges[*ei])
            .flat_map(|edge| vec![edge.0, edge.1])
            .collect(); //would like to not have to collect here
                        //get unique values
        let vertis: Vec<VertIndex> = vertis.into_iter().unique().collect();
        let verts_in_face: Vec<V> = vertis.iter().map(|vi| verts[*vi]).collect();
        let center = barycenter(&verts_in_face);
        center.normalize()
    }
    // we need m n-prisms and n m-prisms
    fn make_face1<V: VectorTrait>(
        i: VertIndex,
        ns: &[VertIndex; 2],
        verts: &[V],
        edges: &[Edge],
    ) -> Face<V> {
        let (m, n) = (ns[0], ns[1]);
        let cap1_edgeis = (0..n).map(|j| j + i * n);
        let cap2_edgeis = (0..n).map(|j| j + ((i + 1) % m) * n);
        let long_edgeis = (0..n).map(|j| m * n + j + i * n);
        let edgeis: Vec<EdgeIndex> = cap1_edgeis.chain(cap2_edgeis).chain(long_edgeis).collect();
        let normal = make_normal(&edgeis, verts, edges);
        Face::new(edgeis, normal, false)
    }
    fn make_face2<V: VectorTrait>(
        j: VertIndex,
        ns: &[VertIndex; 2],
        verts: &[V],
        edges: &[Edge],
    ) -> Face<V> {
        let (m, n) = (ns[0], ns[1]);
        let cap1_edgeis = (0..m).map(|i| m * n + j + i * n);
        let cap2_edgeis = (0..m).map(|i| m * n + (j + 1) % n + i * n);
        let long_edgeis = (0..m).map(|i| j + i * n);
        let edgeis: Vec<EdgeIndex> = cap1_edgeis.chain(cap2_edgeis).chain(long_edgeis).collect();
        let normal = make_normal(&edgeis, verts, edges);
        Face::new(edgeis, normal, false)
    }
    let faces_1 = (0..ns[0]).map(|i| make_face1(i, &ns.clone(), &verts, &edges));
    let faces_2 = (0..ns[1]).map(|j| make_face2(j, &ns.clone(), &verts, &edges));
    let faces: Vec<Face<V>> = faces_1.chain(faces_2).collect();

    // for face in &faces {
    // 	println!("{}",face)
    // }

    Shape::new_convex(verts, edges, faces)
}

pub fn invert_normals<V: VectorTrait>(shape: &Shape<V>) -> Shape<V> {
    let mut new_shape = shape.clone();
    for face in &mut new_shape.faces {
        face.geometry.plane.normal = -face.normal();
    }
    //new_shape.update_from_ref(shape, &Transform::identity());
    new_shape
}
