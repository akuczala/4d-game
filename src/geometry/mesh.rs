
use super::{VectorTrait,VertIndex};

type FacetIndex = VertIndex;

#[derive(Copy,Clone)]
struct Facet0;
impl Facet0 {
    fn new() -> Self {
        todo!()
    }
}
#[derive(Copy,Clone)]
struct Facet1;
impl Facet1 {
    fn new(vi0 : FacetIndex, vi1 : FacetIndex) -> Self {
        todo!()
    }
}
#[derive(Copy,Clone)]
struct Facet2;
#[derive(Copy,Clone)]
struct Facet3;

#[derive(Copy,Clone)]
//needs only contain the number of verts
struct Facets0<V : VectorTrait>(pub V); //no normals
impl<V : VectorTrait> Facets0<V> {
    fn new(f0s : &Vec<Facet0>) -> Self {
        todo!()
    }
}
#[derive(Copy,Clone)]
struct Facets1<V : VectorTrait>(pub V); //only points have normals
impl<V : VectorTrait> Facets1<V> {
    fn new(vert_normals : Vec<V>, edgeis : Vec<Facet1>) -> Self {
        todo!()
    }
}
#[derive(Copy,Clone)]
struct Facets2<V : VectorTrait>(pub V); //only edges and faces have normals
#[derive(Copy,Clone)]
struct Facets3<V : VectorTrait>(pub V); //only faces and volumes have normals

impl<V : VectorTrait> Facets0<V> {
    fn extrude(&self, evec : V) -> Facets1<V> {
        let edgeis : Vec<Facet1> = (0..self.get_n_verts())
            .map(|vi| Facet1::new(vi,vi+self.get_n_verts()))
            .collect();
        let vert_normals = {
            let back = (0..self.get_n_verts())
                .map(|_| -evec.normalize());
            let front = (0..self.get_n_verts())
                .map(|_| evec.normalize());
            back.chain(front).collect()
        };
        Facets1::new(vert_normals,edgeis)
    }
    fn set_vertis(&mut self, vertis : Vec<FacetIndex>) {
        todo!()
    }
    fn get_vertis(&self) -> Vec<FacetIndex> {
        todo!()
    }
    fn get_n_verts(&self) -> FacetIndex {
        todo!()
    }
    fn get_shifted(&self, evec : V) -> Facets0<V> {
        let mut out = self.clone();
        out.set_vertis(
            self.get_vertis().iter()
                .map(|&vi| vi + self.get_n_verts())
                .collect()
        );
        out
    }
}
impl<V : VectorTrait> Facets1<V> {
    fn extrude(&self, evec : V) -> Facets2<V> {
        let edgeis = (0..self.get_n_verts())
            .map(|vi| Facet1::new(vi,vi+self.get_n_verts()));

        let edge_normals = self.get_vert_normals().iter()
            .map(|&n| {
                let mut edge_normal = VectorTrait::cross(vec![evec].into_iter());
                if edge_normal.dot(n) < 0. {
                    edge_normal = -edge_normal;
                
                }
                edge_normal
            });

        //let edges = edgeis.zip(edge_normals)
        //    .map(|&ei,n| )
        todo!()
    }
    fn set_vertis(&mut self, vertis : Vec<FacetIndex>) {
        todo!()
    }
    fn get_vertis(&self) -> Vec<FacetIndex> {
        todo!()
    }
    fn get_n_verts(&self) -> FacetIndex {
        todo!()
    }
    fn get_vert_normals(&self) -> Vec<V> {
        todo!()
    }
    fn get_shifted(&self, evec : V) -> Facets1<V> {
        let mut out = self.clone();
        out.set_vertis(
            self.get_vertis().iter()
                .map(|&vi| vi + self.get_n_verts())
                .collect()
        );
        out
    }
}

pub enum Facets<V : VectorTrait> {
    E0(Facets0<V>), E1(Facets1<V>), E2(Facets2<V>), E3(Facets3<V>)
}
pub struct Mesh<V : VectorTrait> {
    verts : Vec<V>,
    facets : Facets<V>,
}

pub struct MeshBuilder<V :VectorTrait>(Option<Mesh<V>>);
impl<V :VectorTrait> MeshBuilder<V> {
    pub fn point(p : V) -> MeshBuilder<V> {
        Self(Some(
            Mesh{
                verts : vec![p],
                facets : Facets::E0(Facets0::<V>::new(&vec![Facet0::new()])),
            }
        ))
    }
    pub fn extrude(builder : Self,  evec : V) -> Self {
        let mesh = builder.0.expect("No mesh to extrude");
        let facets = mesh.facets;
        let new_facets = todo!();
        let verts = mesh.verts;
        let new_verts = verts.iter().map(|&v| v)
            .chain(verts.iter().map(move |&v| (v + evec)))
            .collect();
        MeshBuilder(Some(Mesh{verts : new_verts, facets : new_facets}))

    }
}