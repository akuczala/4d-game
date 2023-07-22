pub mod bbox;
pub mod systems;

use crate::components::{
    Camera, Convex, Player, Shape, ShapeType, SingleFace, Transform, Transformable,
};
use crate::constants::{PLAYER_COLLIDE_DISTANCE, ZERO};
use crate::ecs_utils::{Componentable, ModSystem};
use crate::geometry::shape::single_face::BoundarySubFace;
use crate::geometry::shape::subface::SubFace;
use crate::geometry::shape::FaceIndex;
use crate::geometry::transform::Scaling;
use crate::geometry::Face;
use crate::input::key_map::PRINT_DEBUG;
use crate::input::Input;
use crate::spatial_hash::{HashInt, SpatialHashSet};
use crate::utils::partial_max;
use crate::vector::{Field, VectorTrait};
use itertools::Itertools;
use specs::prelude::*;
use specs::Component;
use std::marker::PhantomData;

pub use self::bbox::{BBox, HasBBox};
use self::systems::BBoxHashingSystem;

#[derive(Clone, Component)]
#[storage(VecStorage)]
pub struct StaticCollider;

#[derive(Component)]
#[storage(HashMapStorage)]
pub struct InPlayerCell;

pub struct MoveNext<V> {
    pub next_dpos: Option<V>,
    pub can_move: Option<bool>,
}
impl<V> Default for MoveNext<V> {
    fn default() -> Self {
        Self {
            next_dpos: None,
            can_move: None,
        }
    }
}
impl<V: VectorTrait> Transformable<V> for MoveNext<V> {
    fn transform(&mut self, transform: Transform<V, V::M>) {
        self.next_dpos = match self.next_dpos {
            Some(v) => Some(v + transform.pos),
            None => Some(transform.pos),
        };
    }
}

impl<V: VectorTrait> HasBBox<V> for Shape<V> {
    fn calc_bbox(&self) -> BBox<V> {
        let verts = &self.verts;

        //take smallest and largest components to get bounding box
        let (mut min, mut max) = (verts[0], verts[0]);
        for &v in verts.iter() {
            min = min.zip_map(v, Field::min);
            max = max.zip_map(v, Field::max);
        }
        BBox { min, max }
    }
}

pub fn create_spatial_hash<V: VectorTrait + Componentable>(world: &mut World) {
    //add bbox entities and initialize spatial hash set
    let (mut max, mut min) = (V::zero(), V::zero());
    let mut max_lengths = V::zero();
    for bbox in (&world.read_component::<BBox<V>>()).join() {
        min = min.zip_map(bbox.min, Field::min);
        max = max.zip_map(bbox.max, Field::max);
        max_lengths = max_lengths.zip_map(bbox.max - bbox.min, Field::max);
    }
    // set default if there are no finite size bboxes in world
    // if V::is_close(max, min) {
    // 	min = V::ones() * (-50.0);
    // 	max = V::ones() * (50.0);
    // 	max_lengths = V::ones() * 10.0;
    // }
    min = V::ones() * (-50.0);
    max = V::ones() * (50.0);
    max_lengths = V::ones() * 5.0;

    //println!("Min/max: {},{}",min,max);
    //println!("Longest sides {}",max_lengths);
    world.insert(SpatialHashSet::<V, Entity>::new(
        min * 1.5, //make bounds slightly larger than farthest points
        max * 1.5,
        max_lengths * 1.1, //make cell size slightly larger than largest bbox dimensions
    ));
    //enter bboxes into hash set
    //BBoxHashingSystem(ModSystem::typed_default(PhantomData::<V>)).run_now(&world);
}

fn get_bbox_cells<V: VectorTrait>(
    bbox: &BBox<V>,
    hash: &SpatialHashSet<V, Entity>,
) -> Vec<HashInt> {
    let max_coords = hash.get_cell_coords(&bbox.max);
    let min_coords = hash.get_cell_coords(&bbox.min);
    let dcoords: Vec<HashInt> = max_coords
        .iter()
        .zip(min_coords)
        .map(|(max, min)| max - min)
        .collect();
    get_dcoords_dcells(&dcoords, &hash.0.multiplier)
        .into_iter()
        .map(|dc| hash.hash(&bbox.min) + dc) //add min (base) cell
        .collect()
}

//for simplicity, let's assume that every colliding entity has a bbox small enough that it can occupy up to 2^d cells.
//this means that we should have the cell sizes larger than the longest object, for each axis
//this may be problematic if we want to consider entities with size comparable to the scene size - then the hash map is useless

fn get_entities_in_bbox<V: VectorTrait>(
    bbox: &BBox<V>,
    hash: &SpatialHashSet<V, Entity>,
) -> Vec<Entity> {
    get_bbox_cells(bbox, hash)
        .into_iter()
        .filter_map(|cell| hash.get_from_cell(cell)) //get hash set from each cell, if it exists
        .flat_map(|hashset| hashset.iter()) //get iterator from each hash set
        .copied() //deref
        .unique() //remove duplicate entities
        .collect()
}

pub fn insert_static_bbox<V: VectorTrait>(
    hash: &mut SpatialHashSet<V, Entity>,
    bbox: &BBox<V>,
    entity: Entity,
) {
    for cell in get_bbox_cells(bbox, hash) {
        hash.insert_at_cell(cell, entity)
    }
}

pub fn insert_static_bboxes<'a, V: VectorTrait + 'a, I>(
    hash: &mut SpatialHashSet<V, Entity>,
    bbox_entity_iter: I,
) where
    I: Iterator<Item = (&'a BBox<V>, Entity)>,
{
    for (bbox, entity) in bbox_entity_iter {
        insert_static_bbox(hash, bbox, entity)
    }
}

pub fn update_static_bboxes<'a, V: VectorTrait + 'a, I>(
    hash: &mut SpatialHashSet<V, Entity>,
    bbox_entity_iter: I,
) where
    I: Iterator<Item = (&'a BBox<V>, Entity)>,
{
    for (bbox, entity) in bbox_entity_iter {
        hash.remove_from_all(&entity);
        insert_static_bbox(hash, bbox, entity)
    }
}

fn get_bits(n: HashInt, n_bits: HashInt) -> impl Iterator<Item = HashInt> {
    //might be more sensible to be bool?
    (0..n_bits).map(move |k| n.rotate_right(k) % 2)
}
//could memoize results here if too slow. but should be fast
//assumes entries of dcoords are 0 or 1
fn get_dcoords_dcells(dcoords: &[HashInt], mult: &[HashInt]) -> Vec<HashInt> {
    //let dim = dcoords.len();
    assert!(dcoords.iter().all(|&d| d == 0 || d == 1));
    let dpos: Vec<usize> = dcoords
        .iter()
        .enumerate()
        .filter_map(|(i, d)| match d {
            1 => Some(i),
            _ => None,
        })
        .collect();

    let box_dim = dcoords.iter().sum();
    let kmax: HashInt = (2 as HashInt).pow(box_dim);
    let mut out_vec: Vec<HashInt> = vec![];

    for k in 0..kmax {
        let k_bits = get_bits(k, box_dim);
        let cell = k_bits
            .zip(dpos.iter())
            .map(|(kb, &d)| kb * mult[d])
            .sum::<HashInt>();
        //println!("CELL: {:?} vs {:?}",cell,k);
        out_vec.push(cell)
    }
    out_vec
}

pub fn move_player<V: VectorTrait>(
    move_next: &mut MoveNext<V>,
    player_transform: &mut Transform<V, V::M>,
    camera: &mut Camera<V>,
) {
    if let MoveNext {
        next_dpos: Some(next_dpos),
        can_move: Some(true),
    } = move_next
    {
        player_transform.translate(*next_dpos);
        camera.update(player_transform);
    };
    *move_next = MoveNext::default(); //clear movement
}

pub fn update_player_bbox<V: VectorTrait>(player_bbox: &mut BBox<V>, player_pos: V) {
    player_bbox.min = player_pos - V::constant(0.2);
    player_bbox.max = player_pos + V::constant(-0.2);
}

/// finds the max (signed) distance to a face's subface planes
pub fn face_max_subface_dist<V: VectorTrait>(
    shape_subfaces: &[SubFace<V>],
    face_i: FaceIndex,
    pos: V,
) -> Option<Field> {
    let face_boundary_subfaces: Vec<&BoundarySubFace<V>> = shape_subfaces
        .iter()
        .flat_map(|sf| match sf {
            SubFace::Convex(_) => None,
            SubFace::Boundary(bsf) => (bsf.facei == face_i).then_some(bsf),
        })
        .collect();
    partial_max(
        face_boundary_subfaces
            .iter()
            .map(|bsf| bsf.plane.point_signed_distance(pos)),
    )
}

/// returns vec of faces within some distance of the player
/// We currently assume that all faces are one-sided and form a convex hull
pub fn colliding_faces<V: VectorTrait>(
    shape: &Shape<V>,
    collide_distance: Field,
    player_pos: V,
) -> Vec<&Face<V>> {
    // TODO: handle two-sided case
    // TODO: reduce redundant work by integrating this calculation with below
    let (_, convex_dist) = shape.point_normal_distance(player_pos);

    let mut out = Vec::new();
    let subfaces = shape.shape_type.get_subfaces();
    for (face_i, face) in shape.faces.iter().enumerate() {
        let face_dist = face.plane().point_signed_distance(player_pos);
        if (convex_dist < collide_distance)
            && (face_dist > ZERO)
            && (face_dist < collide_distance)
            && face_max_subface_dist(&subfaces, face_i, player_pos)
                .map_or(true, |d| d < collide_distance)
        {
            out.push(face)
        }
    }
    out
}

pub fn check_player_static_collisions<'a, I, V: VectorTrait + 'a>(
    move_next: &mut MoveNext<V>,
    player_pos: V,
    shape_iter: I,
) where
    I: Iterator<Item = &'a Shape<V>>,
{
    if let MoveNext {
        next_dpos: Some(_next_dpos),
        can_move: Some(true),
    } = move_next
    {
        for shape in shape_iter {
            let next_dpos = move_next.next_dpos.unwrap();
            //this is more convoluted than it needs to be
            // for concave shapes, more than one face can push the player away simultaneously (at the corner)
            for face in colliding_faces(shape, PLAYER_COLLIDE_DISTANCE, player_pos) {
                let normal = face.normal();
                //push player away along normal of nearest face (projects out -normal)
                //but i use abs here to guarantee the face always repels the player
                let new_dpos = next_dpos + (normal) * (normal.dot(next_dpos).abs());

                move_next.next_dpos = Some(new_dpos);
                //println!("{}",normal);
            }
        }
    }
}

#[test]
fn dcoords_cells_test() {
    //let result : Vec<HashInt> = (0..8).collect();
    let mult = vec![1, 2, 4];
    //let dcoords = vec![1,1,1];

    assert_eq!(get_dcoords_dcells(&vec![1, 1, 1], &mult), {
        let r: Vec<HashInt> = (0..8).collect();
        r
    });
    assert_eq!(get_dcoords_dcells(&vec![0, 0, 0], &mult), vec![0]);
    assert_eq!(get_dcoords_dcells(&vec![1, 0, 0], &mult), vec![0, 1]);
    assert_eq!(get_dcoords_dcells(&vec![0, 1, 0], &mult), vec![0, 2]);
    assert_eq!(get_dcoords_dcells(&vec![0, 0, 1], &mult), vec![0, 4]);
    assert_eq!(get_dcoords_dcells(&vec![1, 1, 0], &mult), vec![0, 1, 2, 3]);
    assert_eq!(get_dcoords_dcells(&vec![0, 1, 1], &mult), vec![0, 2, 4, 6]);
    assert_eq!(get_dcoords_dcells(&vec![1, 0, 1], &mult), vec![0, 1, 4, 5]);
}
