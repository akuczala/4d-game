
use super::{VectorTrait,VertIndex};

type FacetIndex = VertIndex;

trait FacetTrait : Clone {
    fn map<F : Fn(FacetIndex) -> FacetIndex>(&self, f : F) -> Self;
    // fn shifted(&self, n : FacetIndex) -> Self {
    //     self.map(|x| x + n)
    // }
}
#[derive(Copy,Clone,Debug)]
struct Facet0(FacetIndex);
impl Facet0 {
    fn new(i : FacetIndex) -> Self {
        Self(i)
    }
    
    fn shifted(&self, n : FacetIndex) -> Self {
        self.map(|x| x + n)
    }
}
#[derive(Copy,Clone,Debug)]
struct Facet1(FacetIndex,FacetIndex);
impl Facet1 {
    fn new(vi0 : Facet0, vi1 : Facet0) -> Self {
        Self(vi0.0,vi1.0)
    }
    
    fn shifted(&self, n : FacetIndex) -> Self {
        self.map(|x| x + n)
    }
}
#[derive(Clone,Debug)]
struct Facet2(Vec<FacetIndex>);
impl Facet2 {
    fn new(edgeis : Vec<FacetIndex>) -> Self {
        Facet2(edgeis)
    }
    
    fn shifted(&self, n : FacetIndex) -> Self {
        self.map(|x| x + n)
    }
}
#[derive(Clone,Debug)]
struct Facet3(Vec<FacetIndex>);
impl Facet3 {
    fn new(faceis : Vec<FacetIndex>) -> Self {
        Facet3(faceis)
    }
    
    fn shifted(&self, n : FacetIndex) -> Self {
        self.map(|x| x + n)
    }
}
impl FacetTrait for Facet0 {
    fn map<F : Fn(FacetIndex) -> FacetIndex>(&self, f : F) -> Self {
        Self(f(self.0))
    }
}
impl FacetTrait for Facet1 {
    fn map<F : Fn(FacetIndex) -> FacetIndex>(&self, f : F) -> Self {
        Self(f(self.0),f(self.1))
    }
}
impl FacetTrait for Facet2 {
    fn map<F : Fn(FacetIndex) -> FacetIndex>(&self, f : F) -> Self {
        Self(self.0.iter().map(|&x| f(x)).collect())
    }
}
impl FacetTrait for Facet3 {
    fn map<F : Fn(FacetIndex) -> FacetIndex>(&self, f : F) -> Self {
        Self(self.0.iter().map(|&x| f(x)).collect())
    }
}

#[derive(Clone,Debug)]
struct FacetList<T : FacetTrait>(Vec<T>);
impl<T : FacetTrait> FacetList<T> {
    fn map_index<F : Fn(FacetIndex) -> FacetIndex + Copy>(&self, f : F) -> Self {
        Self(self.0.iter().map(|x| x.map(f)).collect())
    }
    fn concat(&self, other: &FacetList<T>) -> Self {
        let mut new = self.0.clone();
        new.extend(other.0.clone());
        Self(new)
    }
    fn len(&self) -> FacetIndex {
        self.0.len()
    }
    fn extended(&self, other : &Self) -> Self {
        let mut out = self.0.clone();
        out.extend(other.0.clone());
        Self(out)
    }
}
use std::ops::Index;
impl<T : FacetTrait> Index<FacetIndex> for FacetList<T> {
    type Output = T;

    fn index(&self, i: FacetIndex) -> &Self::Output {
        &self.0[i]
    }
}

#[derive(Clone,Debug)]
struct FacetComplex{
    vertis: FacetList<Facet0>,
    edges: FacetList<Facet1>,
    faces: FacetList<Facet2>,
    volumes: FacetList<Facet3>
}
impl FacetComplex{
    fn concat(&self, other: &Self) -> Self {
        let n = self.vertis.len();
        let shifted = self.shifted();
        Self{
            vertis: self.vertis.concat(&other.vertis),
            edges: self.edges.concat(&other.edges),
            faces: self.faces.concat(&other.faces),
            volumes: self.volumes.concat(&other.volumes),
        }
    }
    fn shifted(&self) -> Self {
        Self{
            vertis: self.vertis.map_index(|x| x + self.vertis.len()),
            edges: self.edges.map_index(|x| x + self.vertis.len()),
            faces: self.faces.map_index(|x| x + self.edges.len()),
            volumes: self.volumes.map_index(|x| x + self.faces.len()),
        }
    }
    fn to_string(&self) -> String {
        todo!()
    }
}

pub fn extrude<V : VectorTrait>(mesh : &Mesh<V>, evec : V) -> Mesh<V> {
    let n = mesh.facet_complex.vertis.len();
    let n_edges = mesh.facet_complex.edges.len();
    let n_faces = mesh.facet_complex.faces.len();

    //copy mesh to translated position
    let mut shifted = mesh.translated(evec);
    shifted.facet_complex = shifted.facet_complex.shifted();

    //build new edges, faces, etc between these two meshes
    let long_edges : Vec<Facet1> = mesh.facet_complex.vertis.0.iter()
        .zip(shifted.facet_complex.vertis.0.iter())
        .map(|(&vi0,&vi1)| Facet1::new(vi0,vi1))
        .collect();
    let long_edges = FacetList(long_edges);

    let long_faces : Vec<Facet2> = mesh.facet_complex.edges.0.iter().enumerate()
        .map(|(ei0,&e0)| {
            let long_eis = (2*n_edges + e0.0, 2*n_edges + e0.1);
            Facet2::new(vec![ei0,long_eis.0,ei0 + n_edges,long_eis.1])
        })
        .collect();
    let long_faces = FacetList(long_faces);

    let edges = mesh.facet_complex.edges
        .extended(&shifted.facet_complex.edges)
        .extended(&long_edges);
    let faces = mesh.facet_complex.faces
        .extended(&shifted.facet_complex.faces)
        .extended(&long_faces);
    let volumes = FacetList(vec![]);

    let mut verts =  mesh.verts.clone();
    verts.extend(shifted.verts.clone());

    Mesh{
        verts,
        facet_complex: FacetComplex{
            vertis: mesh.facet_complex.vertis.extended(&shifted.facet_complex.vertis),
            edges, faces, volumes 
        }
    }

}

#[derive(Clone,Debug)]
pub struct Mesh<V : VectorTrait> {
    verts : Vec<V>,
    facet_complex : FacetComplex,
}
impl<V: VectorTrait> Mesh<V> {
    pub fn translated(&self, v: V) -> Self {
        let mut new = Mesh{ verts: self.verts.clone(), facet_complex: self.facet_complex.clone()};
        new.translate(v);
        new
    }
    pub fn translate(&mut self, v: V) {
        for vert in self.verts.iter_mut() {
            *vert = *vert + v;
        }
    }
    pub fn concat(&self, other: &Self) -> Self {
        Self{
            verts: {let mut new = self.verts.clone(); new.extend(other.verts.clone()); new},
            facet_complex: self.facet_complex.concat(&other.facet_complex),
        }
    }
}

#[test]
fn test_extrude() {
    use crate::vector::{Vec3};
    let point = Mesh{
        verts: vec![Vec3::new(0.,0.,0.)],
        facet_complex : FacetComplex{
            vertis: FacetList(vec![Facet0::new(0)]),
            edges: FacetList(vec![]),
            faces: FacetList(vec![]),
            volumes: FacetList(vec![]),
        }
    };
    let line = extrude(&point,Vec3::new(1.0,0.,0.));
    let square = extrude(&line,Vec3::new(0.0,1.,0.));
    let cube = extrude(&square,Vec3::new(0.,0.,1.0));
    println!("point");
    println!("{:?}", point);
    println!("line");
    println!("{:?}", line);
    println!("square");
    println!("{:?}", square);
    println!("cube");
    println!("{:?}", cube);
    assert!(false)
}