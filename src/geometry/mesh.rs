
use super::{VectorTrait,MatrixTrait,VertIndex};
use std::fmt;

type FacetIndex = VertIndex;

trait FacetTrait :  Clone + fmt::Display {
    fn map<F : Fn(FacetIndex) -> FacetIndex>(&self, f : F) -> Self;
}

#[derive(Clone,Debug)]
struct Facet0<V: VectorTrait>{i: FacetIndex, normal: V}
impl<V: VectorTrait> Facet0<V> {
    fn new(i : FacetIndex, normal: V) -> Self {
        Self{i, normal}
    }
}
#[derive(Clone,Debug)]
struct Facet1<V: VectorTrait>{edge: (FacetIndex,FacetIndex), normal: V}
impl<V: VectorTrait> Facet1<V> {
    fn new(vi0 : FacetIndex, vi1 : FacetIndex, normal: V) -> Self {
        Self{edge: (vi0,vi1), normal}
    }
}
#[derive(Clone,Debug)]
struct Facet2<V: VectorTrait>{edgeis: Vec<FacetIndex>, normal: V}
impl<V: VectorTrait> Facet2<V> {
    fn new(edgeis : Vec<FacetIndex>, normal: V) -> Self {
        Self{edgeis, normal}
    }
    fn get_vec(&self) -> &Vec<FacetIndex> {
        &self.edgeis
    }
}
#[derive(Clone,Debug)]
struct Facet3<V: VectorTrait>{faceis: Vec<FacetIndex>, normal: V}
impl<V: VectorTrait> Facet3<V> {
    fn new(faceis : Vec<FacetIndex>, normal: V) -> Self {
        Facet3{faceis, normal}
    }
}

impl<V: VectorTrait> FacetTrait for Facet0<V> {
    fn map<F : Fn(FacetIndex) -> FacetIndex>(&self, f : F) -> Self {
        Self{i: f(self.i), normal: self.normal}
    }
}
impl<V: VectorTrait> FacetTrait for Facet1<V> {
    fn map<F : Fn(FacetIndex) -> FacetIndex>(&self, f : F) -> Self {
        Self{edge: (f(self.edge.0),f(self.edge.1)), normal: self.normal}
    }
}
impl<V: VectorTrait> FacetTrait for Facet2<V> {
    fn map<F : Fn(FacetIndex) -> FacetIndex>(&self, f : F) -> Self {
        Self{edgeis: self.edgeis.iter().map(|&x| f(x)).collect(), normal: self.normal}
    }
}
impl<V: VectorTrait> FacetTrait for Facet3<V> {
    fn map<F : Fn(FacetIndex) -> FacetIndex>(&self, f : F) -> Self {
        Self{faceis: self.faceis.iter().map(|&x| f(x)).collect(), normal: self.normal}
    }
}
fn list_string<'a, T: 'a + fmt::Display + Copy>(iter: impl Iterator<Item = &'a T>) -> String {
    let mut out = format!("");
    for &v in iter {
        out = format!("{}{} ",out,v);
    }
    out
}
fn list_stack_string<'a, T: 'a + fmt::Display>(iter: impl Iterator<Item = &'a T>) -> String {
    let mut out = format!("");
    for v in iter { 
        out = format!("{}{}\n",out,v);
    }
    out
}

impl<V : VectorTrait> fmt::Display for Facet0<V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // let mut vec_str = format!("");
        // for &n_i in self.normal.iter() {
        //     vec_str = format!("{} ",n_i)
        // }
        let vec_str = list_string(self.normal.iter());
        write!(f, "[{}], [{}]",self.i,vec_str)
    }
}
impl<V : VectorTrait> fmt::Display for Facet1<V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let vec_str = list_string(self.normal.iter());
        write!(f, "[{},{}], [{}]",self.edge.0,self.edge.1,vec_str)
    }
}
impl<V : VectorTrait> fmt::Display for Facet2<V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let vec_str = list_string(self.normal.iter());
        let i_str = list_string(self.edgeis.iter());
        write!(f, "[{}], [{}]",i_str,vec_str)
    }
}
impl<V : VectorTrait> fmt::Display for Facet3<V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let vec_str = list_string(self.normal.iter());
        let i_str = list_string(self.faceis.iter());
        write!(f, "[{}], [{}]",i_str,vec_str)
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
    fn map<F : Fn(&T) -> T>(&self, f: F) -> Self {
        Self(self.0.iter().map(|facet| f(facet)).collect())
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
impl<T: FacetTrait> fmt::Display for FacetList<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let vec_str = list_stack_string(self.0.iter());
        write!(f, "{}",vec_str)
    }
}

fn make_perp_towards<V: VectorTrait>(perp_to: V, towards: V) -> V {
    let perp_to_norm = perp_to.normalize();
    (towards - perp_to_norm*towards.dot(perp_to_norm)).normalize()
} 
fn reflect_along<V : VectorTrait>(v: V, along: V) -> V {
    v - along*2.0*v.dot(along.normalize())
}
// fn cross_along()
// CrossAlong[v1_, v2_, along_] := Module[{cross = Cross[v1, v2]},
//    If[cross.along > 0, cross, -cross]
//    ];

#[derive(Clone,Debug)]
struct FacetComplex<V: VectorTrait>{
    vertis: FacetList<Facet0<V>>,
    edges: FacetList<Facet1<V>>,
    faces: FacetList<Facet2<V>>,
    volumes: FacetList<Facet3<V>>
}
impl<V: VectorTrait> FacetComplex<V>{
    fn empty() -> Self {
        Self{
            vertis: FacetList::<Facet0<V>>::empty(),
            edges: FacetList::<Facet1<V>>::empty(),
            faces: FacetList::<Facet2<V>>::empty(),
            volumes: FacetList::<Facet3<V>>::empty(),
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
impl<V: VectorTrait> fmt::Display for FacetComplex<V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VERTS\n{}\nEDGES\n{}\nFACES\n{}\nVOLUMES\n{}\n",self.vertis, self.edges, self.faces, self.volumes)
    }
}
//replicate mesh-test-p2.nb (in progress)
//need to change the order of normal assignment

pub fn extrude<V : VectorTrait>(mesh : &Mesh<V>, evec : V) -> Mesh<V> {
    let placeholder = V::zero();
    let facets = &mesh.facet_complex;
    let shifted_facets = mesh.facet_complex.shifted();

    let mut new_facets = FacetComplex::empty();
    let vertis_far = new_facets.vertis.extend(
        &facets.vertis.map(|facet| Facet0::new(facet.i, -evec.normalize()))
    );
    let vertis_close = new_facets.vertis.extend(
        &shifted_facets.vertis.map(|facet| Facet0::new(facet.i, evec.normalize()))
    );

    let far_edgeis = new_facets.edges.extend(&facets.edges.clone());
    let close_edgeis = new_facets.edges.extend(&shifted_facets.edges.clone());
    let long_edgeis = new_facets.edges.extend(&FacetList(
        vertis_far.iter()
            .zip(vertis_close.iter())
            .map(|(&far_i,&close_i)| Facet1::new(far_i,close_i, make_perp_towards(evec, new_facets.vertis[far_i].normal)))
            .collect()
    ));

    let far_faceis = new_facets.faces.extend(&facets.faces.clone());
    let close_faceis = new_facets.faces.extend(&shifted_facets.faces.clone());
    let long_faceis = new_facets.faces.extend(&FacetList(
        far_edgeis.iter().zip(close_edgeis.iter())
        .map(|(&far_i,&close_i)| Facet2::new(vec![
                far_i,
                long_edgeis[new_facets.edges[far_i].edge.1],
                close_i,
                long_edgeis[new_facets.edges[far_i].edge.0]
            ], placeholder))
        .collect()
        ));

    let _close_voluis = new_facets.volumes.extend(&facets.volumes.clone());
    let _far_voluis = new_facets.volumes.extend(&shifted_facets.volumes.clone());
    let _long_voluis = new_facets.volumes.extend(&FacetList(
        far_faceis.iter().zip(close_faceis.iter())
        .map(|(&far_i,&close_i)| Facet3::new(
                vec![far_i, close_i].into_iter()
                .chain(
                    new_facets.faces[far_i].get_vec().iter()
                    .map(|&ei| long_faceis[ei])
                )
                .collect(), placeholder
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
    facet_complex : FacetComplex<V>,
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
impl<V: VectorTrait> fmt::Display for Mesh<V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}",self.facet_complex)
    }
}


#[test]
fn test_extrude() {
    use crate::vector::{Vec3,Vec4};
    type VecN = Vec3;
    let point = Mesh{
        verts: vec![VecN::zero()],
        facet_complex : FacetComplex{
            vertis: FacetList(vec![Facet0::new(0, VecN::ones().normalize())]),
            edges: FacetList(vec![]),
            faces: FacetList(vec![]),
            volumes: FacetList(vec![]),
        }
    };
    let id_mat = <VecN as VectorTrait>::M::id();
    let line = extrude(&point,id_mat[0]);
    let square = extrude(&line,id_mat[1]);
    let cube = extrude(&square,id_mat[2]);
    //let tess = extrude(&cube,Vec4::new(0.,0.,0.,1.0));
    println!("point");
    println!("{}", point);
    println!("line");
    println!("{}", line);
    println!("square");
    println!("{}", square);
    println!("cube");
    println!("{}", cube);
    //println!("tess");
    //println!("{}", tess);
    assert!(false)
}