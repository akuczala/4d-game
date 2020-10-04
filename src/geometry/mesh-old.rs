use itertools::Itertools;
use std::hash::Hash;
use super::{VectorTrait,VertIndex};

//properties that the "index" I must satisfy
pub trait FacetI : Copy + Eq + Hash {}

//Facet1 is an edge. it holds two indices
#[derive(Clone,PartialEq,Eq,Hash)]
pub struct Facet1<I : FacetI>(I, I);
//Facet2 is a 2d face. it holds indices corresponding to its points,
//as well as its edges, each of which hold an index corresponding to its points
#[derive(Clone)]
pub struct Facet2<I : FacetI>(Vec<I>,Vec<Facet1<I>>);
pub struct Facet3<I : FacetI>(Vec<I>,Vec<Facet1<I>>,Vec<Facet2<I>>);

impl<I : FacetI> Facet2<I> {
    fn new(edges : &Vec<Facet1<I>>) -> Self {
        let vertis : Vec<I> = edges.iter().map(|f1| vec![f1.0,f1.1])
            .flatten()
            .unique()
            .collect();
        Self(vertis,edges.clone())
    }
}
impl<I : FacetI> Facet3<I> {
    fn new(faces : &Vec<Facet2<I>>) -> Self {
        let vertis : Vec<I> = faces.iter().map(|f2| f2.0.clone())
            .flatten()
            .unique()
            .collect();
        let edges : Vec<Facet1<I>> = faces.iter().map(|f2| f2.1.clone())
            .flatten()
            .unique()
            .collect();
        Self(vertis,edges,(*faces).clone())
    }
}

impl<I : FacetI> Facet1<I> {
    pub fn map<F : Fn(I) -> I + Copy>(&self, f : F) -> Self {
        Self(f(self.0),f(self.1))
    }
}
impl<I : FacetI> Facet2<I> {
    pub fn map<F : Fn(I) -> I + Copy>(&self, f : F) -> Self {
        Self(
            self.0.iter().map(|&i| i).map(f).collect(),
            self.1.iter().map(|f1| f1.map(f)).collect(),
            )
    }
}
impl<I : FacetI> Facet3<I> {
    pub fn map<F : Fn(I) -> I + Copy>(&self, f : F) -> Self {
        Self(
            self.0.iter().map(|&i| i).map(f).collect(),
            self.1.iter().map(|f1| f1.map(f)).collect(),
            self.2.iter().map(|f1| f1.map(f)).collect(),
            )
    }
}

pub enum Facets<I : FacetI> {
    E0,
    E1(Vec<Facet1<I>>),
    E2(Vec<Facet1<I>>,Vec<Facet2<I>>),
    E3(Vec<Facet1<I>>,Vec<Facet2<I>>,Vec<Facet3<I>>),
}
impl<I : FacetI> Facets<I> {
    pub fn map<F : Fn(I) -> I + Copy>(&self, f : F) -> Self {
        match self {
            Self::E0 => Self::E0,
            Self::E1(edges) => Self::E1(
                    edges.iter().map(|e| e.map(f)).collect()
                ),
            Self::E2(edges1,edges2) => Self::E2(
                    edges1.iter().map(|e| e.map(f)).collect(),
                    edges2.iter().map(|e| e.map(f)).collect(),
                ),
            Self::E3(edges1,edges2,edges3) => Self::E3(
                    edges1.iter().map(|e| e.map(f)).collect(),
                    edges2.iter().map(|e| e.map(f)).collect(),
                    edges3.iter().map(|e| e.map(f)).collect(),
                ),
        }
    }
}

#[derive(Clone,Copy,PartialEq,Eq,Hash)]
pub enum Parity {
    Neg, Pos
}
impl Parity {
    fn flip(self) -> Self {
        match self {
            Self::Pos => Self::Neg,
            Self::Neg => Self::Pos,
        }
    }
    fn to_sign(self) -> i8 {
        match self {
            Self::Pos => 1,
            Self::Neg => -1,
        }
    }
    fn from_sign(s : i8) -> Self {
        match s {
            s if s > 0 => Self::Pos,
            s if s < 0 => Self::Neg,
            _ => panic!("Invalid integer for from_sign"),
        }
    }
    fn times(self, par2 : Self) -> Self {
        Parity::from_sign(self.to_sign()*par2.to_sign())
    }
}
#[derive(Clone,Copy,PartialEq,Eq,Hash)]
pub struct FacetInfo {
    index : VertIndex,
    parity : Parity, 
}
impl FacetI for FacetInfo {}
impl FacetInfo {
    fn extrude(self,n : VertIndex) -> Self {
        Self{index : self.index + n, parity : self.parity.flip()}
    }
}

pub struct Mesh<V: VectorTrait> {
    pub verts : Vec<V>,
    pub facets : Facets<FacetInfo>,
}
pub struct MeshBuilder<V :VectorTrait>(Option<Mesh<V>>);
impl<V :VectorTrait> MeshBuilder<V> {
    pub fn point(p : V) -> MeshBuilder<V> {
        Self(Some(Mesh{verts : vec![p], facets : Facets::E0}))
    }
    pub fn extrude(builder : Self,  evec : V) -> Self {
        let mesh = builder.0.expect("No mesh to extrude");
        let facets = mesh.facets;
        let new_facets = match facets {
            Facets::E0 => Facets::E1(vec![Facet1(
                FacetInfo{index : 0, parity : Parity::Neg},
                FacetInfo{index : 1, parity : Parity::Pos}
            )]),
            Facets::E1(edges) => {
                let shifted_edges : Vec<Facet1<FacetInfo>> = edges.iter()
                    .map(|e| e.map(|i| i.extrude(edges.len()))).collect();

                let faces = edges.iter().zip(shifted_edges.iter())
                    .map(|(e0,e1)| Facet2::new(&vec![
                        e0.clone(),Facet1(e0.1,e1.0),e1.clone(),Facet1(e1.1,e0.0)
                        ]))
                    .collect();

                let mut new_edges = edges.clone();
                new_edges.extend(shifted_edges);

                Facets::E2(new_edges, faces)
            },
            Facets::E2(edges,faces) => {
                let shifted_edges : Vec<Facet1<FacetInfo>> = edges.iter()
                    .map(|e| e.map(|i| i.extrude(edges.len()))).collect();
                let shifted_faces : Vec<Facet2<FacetInfo>> = faces.iter()
                    .map(|f| e.map(|i| i.extrude(faces.len()))).collect();
                let volumes = edges.iter().zip(shifted_edges.iter())
                    .map(|(e0,e1)| Facet2::new(&vec![
                        e0.clone(),Facet1(e0.1,e1.0),e1.clone(),Facet1(e1.1,e0.0)
                        ]))
                    .collect();
            },
            _ => todo!()
        };
        let verts = mesh.verts;
        let new_verts = verts.iter().map(|&v| v)
            .chain(verts.iter().map(move |&v| (v + evec)))
            .collect();
        MeshBuilder(Some(Mesh{verts : new_verts, facets : new_facets}))

    }
}