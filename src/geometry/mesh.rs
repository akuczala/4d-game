
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
    fn get_vec(&self) -> &Vec<FacetIndex> {
        &self.0
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
    fn empty() -> Self {
        Self(vec![])
    }
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
    fn extend(&mut self, other : &FacetList<T>) -> Vec<FacetIndex> {
        let n_old = self.len();
        self.0.extend(other.0.clone());
        (n_old..self.len()).collect() // return new indices
    }
    fn get_vec(&self) -> &Vec<T> {
        &self.0
    }
    fn get_facets(&self, indices: &Vec<FacetIndex>) -> Vec<T> {
        indices.iter().map(|&i| self.0[i].clone()).collect()
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
    fn empty() -> Self {
        Self{
            vertis: FacetList::<Facet0>::empty(),
            edges: FacetList::<Facet1>::empty(),
            faces: FacetList::<Facet2>::empty(),
            volumes: FacetList::<Facet3>::empty(),
        }

    }
    fn concat(&self, other: &Self) -> Self {
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
//replicate mesh-test-p2.nb (in progress)

pub fn extrude<V : VectorTrait>(mesh : &Mesh<V>, evec : V) -> Mesh<V> {
    let facets = &mesh.facet_complex;
    let shifted_facets = mesh.facet_complex.shifted();

    let mut new_facets = FacetComplex::empty();
    let vertis_far = new_facets.vertis.extend(&facets.vertis);
    let vertis_close = new_facets.vertis.extend(&shifted_facets.vertis);

    let far_edgeis = new_facets.edges.extend(&facets.edges.clone());
    let close_edgeis = new_facets.edges.extend(&shifted_facets.edges.clone());
    let long_edgeis = new_facets.edges.extend(&FacetList(
        vertis_far.iter()
            .zip(vertis_close.iter())
            .map(|(&f,&sf)| Facet1(f,sf))
            .collect()
    ));

    let far_faceis = new_facets.faces.extend(&facets.faces.clone());
    let close_faceis = new_facets.faces.extend(&shifted_facets.faces.clone());
    let long_faceis = new_facets.faces.extend(&FacetList(
        far_edgeis.iter().zip(close_edgeis.iter())
        .map(|(&far_i,&close_i)| Facet2(vec![
                far_i,
                long_edgeis[new_facets.edges[far_i].1],
                close_i,
                long_edgeis[new_facets.edges[far_i].0]
            ]))
        .collect()
        ));

    let _close_voluis = new_facets.volumes.extend(&facets.volumes.clone());
    let _far_voluis = new_facets.volumes.extend(&shifted_facets.volumes.clone());
    let _long_voluis = new_facets.volumes.extend(&FacetList(
        far_faceis.iter().zip(close_faceis.iter())
        .map(|(&far_i,&close_i)| Facet3(
                vec![far_i, close_i].into_iter()
                .chain(
                    new_facets.faces[far_i].get_vec().iter()
                    .map(|&ei| long_faceis[ei])
                )
                .collect()
            ))
        .collect()
        ));

    Mesh{
        verts: mesh.verts.clone().into_iter().chain(
                mesh.verts.iter().map(|&v| v + evec)
            ).collect(),
        facet_complex: new_facets
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
    use crate::vector::{Vec4};
    let point = Mesh{
        verts: vec![Vec4::new(0.,0.,0.,0.)],
        facet_complex : FacetComplex{
            vertis: FacetList(vec![Facet0::new(0)]),
            edges: FacetList(vec![]),
            faces: FacetList(vec![]),
            volumes: FacetList(vec![]),
        }
    };
    let line = extrude(&point,Vec4::new(1.0,0.,0.,0.));
    let square = extrude(&line,Vec4::new(0.0,1.,0.,0.));
    let cube = extrude(&square,Vec4::new(0.,0.,1.0,0.));
    let tess = extrude(&cube,Vec4::new(0.,0.,0.,1.0));
    println!("point");
    println!("{:?}", point);
    println!("line");
    println!("{:?}", line);
    println!("square");
    println!("{:?}", square);
    println!("cube");
    println!("{:?}", cube);
    println!("tess");
    println!("{:?}", tess);
    assert!(false)
}