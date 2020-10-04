use std::marker::PhantomData;
use itertools::Itertools;
use std::hash::Hash;
use super::{VectorTrait,VertIndex};

//properties that the "index" I must satisfy
pub trait FacetI : Copy + Eq + Hash {
    fn extrude(self,n : VertIndex);
}
pub trait FacetData : Copy {}
impl<V : VectorTrait> FacetData for V {}

//pub struct Facet0<T : FacetData>(T);

#[derive(Clone,PartialEq,Eq,Hash)]
pub struct Facet0<I : FacetI>(pub PhantomData<I>);
//Facet1 is an edge. it holds two indices
#[derive(Clone,PartialEq,Eq,Hash)]
pub struct Facet1<I : FacetI>(I, I);
//Facet2 is a 2d face. it holds indices corresponding to its points,
//as well as its edges, each of which hold an index corresponding to its points
#[derive(Clone)]
pub struct Facet2<I : FacetI>(Vec<I>,Vec<Facet1<I>>);
pub struct Facet3<I : FacetI>(Vec<I>,Vec<Facet1<I>>,Vec<Facet2<I>>);

pub trait FacetTrait<I : FacetI> {
    type SubFacet : FacetTrait<I>;
    //fn new(indices : &Vec<FacetI>) -> Self;
    fn map<F : Fn(I) -> I + Copy>(&self, f : F) -> Self;
}
impl<I : FacetI> FacetTrait<I> for Facet0<I> {
    type SubFacet = Facet0<I>;
    // fn new(faces : &Vec<FacetI>) -> Self
    // {
    //     Self(PhantomData::<I>)
    // }
    fn map<F : Fn(I) -> I + Copy>(&self, f : F) -> Self {
        self.clone()
    }
}
impl<I : FacetI> FacetTrait<I> for Facet1<I> {
    type SubFacet = Facet0<I>;
    // fn new(faces : &Vec<Self::SubFacet>) -> Self {

    //     let vertis : Vec<I> = faces.iter().map(|f2| f2.0.clone())
    //         .flatten()
    //         .unique()
    //         .collect();
    //     let edges : Vec<Facet1<I>> = faces.iter().map(|f2| f2.1.clone())
    //         .flatten()
    //         .unique()
    //         .collect();
    //     Self(vertis,edges,(*faces).clone())
    // }
    fn map<F : Fn(I) -> I + Copy>(&self, f : F) -> Self {
        Self(f(self.0),f(self.1))
    }
}
impl<I : FacetI> FacetTrait<I> for Facet2<I> {
    type SubFacet = Facet1<I>;
    // fn new(edges : &Vec<Facet1<I>>) -> Self {
    //     let vertis : Vec<I> = edges.iter().map(|f1| vec![f1.0,f1.1])
    //         .flatten()
    //         .unique()
    //         .collect();
    //     Self(vertis,edges.clone())
    // }
    fn map<F : Fn(I) -> I + Copy>(&self, f : F) -> Self {
        Self(
            self.0.iter().map(|&i| i).map(f).collect(),
            self.1.iter().map(|f1| f1.map(f)).collect(),
            )
    }
}
impl<I : FacetI> FacetTrait<I> for Facet3<I> {
    type SubFacet = Facet2<I>;
    fn map<F : Fn(I) -> I + Copy>(&self, f : F) -> Self {
        Self(
            self.0.iter().map(|&i| i).map(f).collect(),
            self.1.iter().map(|f1| f1.map(f)).collect(),
            self.2.iter().map(|f1| f1.map(f)).collect(),
            )
    }
}

struct WithData<T : Copy, F>
{
    data : T,
    facet : F,
}
impl<T : Copy, I : FacetI, J : FacetI> WithData<T,FacetTrait<I,SubFacet=J>> {
    fn map<F : Fn(I) -> I + Copy>(&self, f : F) -> Self {
        Self{data : self.data, facet : f(self.facet)}
    }
}

pub enum Facets<T : FacetData, I : FacetI> {
    E0{vert_data : Vec<T>},
    E1{
        vert_data : Vec<WithData<T,Facet0<I>>>,
        edgeis : Vec<WithData<T,Facet1<I>>>,
    },
    E2{
        vert_data : Vec<T>,
        edgeis : Vec<WithData<T,Facet1<I>>>,
        faceis : Vec<WithData<T,Facet2<I>>>,
    },
    E3{
        vert_data : Vec<T>,
        edgeis : Vec<WithData<T,Facet1<I>>>,
        faceis : Vec<WithData<T,Facet2<I>>>,
        voluis : Vec<WithData<T,Facet3<I>>>,
    },
}
impl<T : FacetData, I : FacetI> Facets<T, I> {
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

type FacetIndex = usize;
impl FacetI for FacetIndex {
    fn extrude(self,n : VertIndex) -> Self {
        self + n
    }
}

pub struct Mesh<V: VectorTrait> {
    pub verts : Vec<V>,
    pub facets : Facets<V, FacetIndex>,
}
// pub struct MeshBuilder<V :VectorTrait>(Option<Mesh<V>>);
// impl<V :VectorTrait> MeshBuilder<V> {
//     pub fn point(p : V) -> MeshBuilder<V> {
//         Self(Some(Mesh{verts : vec![p], facets : Facets::E0}))
//     }
//     pub fn extrude(builder : Self,  evec : V) -> Self {
//         let mesh = builder.0.expect("No mesh to extrude");
//         let facets = mesh.facets;
//         let new_facets = match facets {
//             Facets::E0 => Facets::E1(vec![Facet1(
//                 FacetInfo{index : 0, parity : Parity::Neg},
//                 FacetInfo{index : 1, parity : Parity::Pos}
//             )]),
//             Facets::E1(edges) => {
//                 let shifted_edges : Vec<Facet1<FacetInfo>> = edges.iter()
//                     .map(|e| e.map(|i| i.extrude(edges.len()))).collect();

//                 let faces = edges.iter().zip(shifted_edges.iter())
//                     .map(|(e0,e1)| Facet2::new(&vec![
//                         e0.clone(),Facet1(e0.1,e1.0),e1.clone(),Facet1(e1.1,e0.0)
//                         ]))
//                     .collect();

//                 let mut new_edges = edges.clone();
//                 new_edges.extend(shifted_edges);

//                 Facets::E2(new_edges, faces)
//             },
//             Facets::E2(edges,faces) => {
//                 let shifted_edges : Vec<Facet1<FacetInfo>> = edges.iter()
//                     .map(|e| e.map(|i| i.extrude(edges.len()))).collect();
//                 let shifted_faces : Vec<Facet2<FacetInfo>> = faces.iter()
//                     .map(|f| e.map(|i| i.extrude(faces.len()))).collect();
//                 let volumes = edges.iter().zip(shifted_edges.iter())
//                     .map(|(e0,e1)| Facet2::new(&vec![
//                         e0.clone(),Facet1(e0.1,e1.0),e1.clone(),Facet1(e1.1,e0.0)
//                         ]))
//                     .collect();
//             },
//             _ => todo!()
//         };
//         let verts = mesh.verts;
//         let new_verts = verts.iter().map(|&v| v)
//             .chain(verts.iter().map(move |&v| (v + evec)))
//             .collect();
//         MeshBuilder(Some(Mesh{verts : new_verts, facets : new_facets}))

//     }
// }