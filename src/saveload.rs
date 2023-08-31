use std::fs;
use std::path::Path;

use ron::error::SpannedResult;
use ron::ser::to_string_pretty;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use specs::prelude::*;
use transform::{Transform, Transformable};

use crate::build_level::init_player;
use crate::coin::Coin;
use crate::components::{Heading, Player, RefShapes, ShapeLabel, StaticCollider};

use crate::draw::texture::ShapeTextureBuilder;

use crate::{
    ecs_utils::Componentable, geometry::transform, shape_entity_builder::ShapeEntityBuilderV,
    vector::VectorTrait,
};

// TODO: practice macros?
type LevelSaveComponents<'a, V, M> = (
    ReadStorage<'a, ShapeLabel>,
    ReadStorage<'a, Transform<V, M>>,
    ReadStorage<'a, ShapeTextureBuilder>,
    ReadStorage<'a, StaticCollider>,
    ReadStorage<'a, Coin>,
);
type LevelSaveComponentsV<'a, V> = LevelSaveComponents<'a, V, <V as VectorTrait>::M>;

type EntitySave<V, M> = (
    ShapeLabel,
    Transform<V, M>,
    ShapeTextureBuilder,
    Option<StaticCollider>,
    Option<Coin>,
);

#[derive(Clone, Serialize, Deserialize)]
struct PlayerData<V, M> {
    transform: Transform<V, M>,
    heading: Heading<M>,
}
impl<V> PlayerData<V, V::M>
where
    V: VectorTrait + Componentable,
    V::M: Componentable,
    V::SubV: Componentable,
{
    fn build(world: &World) -> Self {
        let player_entity = world.fetch::<Player>().0;
        let transforms = world.read_storage();
        let headings = world.read_storage::<Heading<V::M>>();
        Self {
            transform: *transforms.get(player_entity).unwrap(),
            heading: *headings.get(player_entity).unwrap(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
struct SaveStructure<V, M> {
    components: Vec<EntitySave<V, M>>,
    player_data: PlayerData<V, M>,
}
type SaveStructureV<V> = SaveStructure<V, <V as VectorTrait>::M>;

/// Clones components into a stucture for serialization
fn build_save_structure<V>(world: &World) -> SaveStructureV<V>
where
    V: VectorTrait + Componentable,
    V::M: Componentable,
    V::SubV: Componentable,
{
    let (labels, transforms, textures, static_colliders, coins) =
        world.system_data::<LevelSaveComponentsV<V>>();
    let components = (
        &labels,
        &transforms,
        &textures,
        (&static_colliders).maybe(),
        (&coins).maybe(),
    )
        .join()
        .map(|(label, transform, texture, maybe_collider, maybe_coin)| {
            (
                label.clone(),
                *transform,
                texture.clone(),
                maybe_collider.cloned(),
                maybe_coin.copied(),
            )
        })
        .collect();
    SaveStructure {
        components,
        player_data: PlayerData::build(world),
    }
}

fn serialize_level<V, M>(save_struct: &SaveStructure<V, M>) -> Result<String, ron::Error>
where
    V: Serialize,
    M: Serialize,
{
    to_string_pretty(save_struct, Default::default())
}

fn deserialize_level<V, M>(level_str: &str) -> SpannedResult<SaveStructure<V, M>>
where
    V: DeserializeOwned,
    M: DeserializeOwned,
{
    ron::from_str(level_str)
}

fn append_components<V>(
    save_struct: SaveStructureV<V>,
    ref_shapes: &RefShapes<V>,
    world: &mut World,
) where
    V: Componentable + VectorTrait,
    V::M: Componentable,
    V::SubV: Componentable,
{
    for (label, transform, texture, maybe_collider, maybe_coin) in save_struct.components {
        ShapeEntityBuilderV::<V>::new(label)
            .with_transform(transform)
            .with_texture(texture)
            .with_collider(maybe_collider)
            .build(ref_shapes, world)
            .maybe_with(maybe_coin)
            .build();
    }
}

pub fn save_level_to_file<V>(path: &Path, world: &mut World) -> std::result::Result<(), ()>
where
    V: Componentable + VectorTrait + Serialize,
    V::M: Componentable + Serialize,
    V::SubV: Componentable + Serialize,
{
    save_save_struct_to_file(path, &build_save_structure::<V>(world))
}

pub fn load_level_from_file<V>(
    path: &str,
    ref_shapes: &RefShapes<V>,
    world: &mut World,
) -> std::result::Result<(), ()>
where
    V: VectorTrait + DeserializeOwned + Componentable,
    V::M: DeserializeOwned + Componentable,
    V::SubV: DeserializeOwned + Componentable,
{
    let save_struct = load_save_struct_from_file(path)?;
    init_player(
        world,
        Some(save_struct.player_data.transform),
        Some(save_struct.player_data.heading),
    );
    append_components(save_struct, ref_shapes, world);
    Ok(())
}

fn load_save_struct_from_file<V: DeserializeOwned, M: DeserializeOwned>(
    path: &str,
) -> Result<SaveStructure<V, M>, ()> {
    std::fs::read_to_string(path)
        .map_err(|e| println!("Error loading level {}: {}", path, e))
        .and_then(|s| {
            deserialize_level::<V, M>(&s).map_err(|e| println!("Could not parse level file: {}", e))
        })
}

fn save_save_struct_to_file<V, M>(
    path: &Path,
    save_struct: &SaveStructure<V, M>,
) -> std::result::Result<(), ()>
where
    V: Serialize,
    M: Serialize,
{
    serialize_level::<V, M>(save_struct)
        .map_err(|e| println!("Could not serialize level: {}", e))
        .and_then(|s| {
            fs::write(path, s)
                .map_err(|e| println!("Could not save to {}: {}", path.to_str().unwrap(), e))
        })
}

// I used the below code to update the rep of Vec4 from a record object to a singleton object
// may or may not be useful in the future, but I anticipate making future changes to the data representation
// this could be used in tandem with creating invariant structs for each datatype for saving, and converting
// the runtime types (which may change over time) to the invariant types
use crate::vector::{Field, Mat4, Vec4};

#[derive(Deserialize, Clone, Copy)]
struct OldVec4 {
    arr: [Field; 4],
}

#[derive(Deserialize, Clone, Copy)]
struct OldMat4(OldVec4, OldVec4, OldVec4, OldVec4);

#[allow(dead_code)]
fn upgrade_savefile() {
    let old_save_struct = load_save_struct_from_file::<OldVec4, OldMat4>("./old.4d.ron").unwrap();
    save_save_struct_to_file(Path::new("./new.4d.ron"), &map_save_struct(old_save_struct)).unwrap();
}

fn convert_vec(v: OldVec4) -> Vec4 {
    Vec4::from_arr(&v.arr)
}

fn convert_mat(m: OldMat4) -> Mat4 {
    Mat4::from_vecs(
        convert_vec(m.0),
        convert_vec(m.1),
        convert_vec(m.2),
        convert_vec(m.3),
    )
}

fn map_save_struct(old: SaveStructure<OldVec4, OldMat4>) -> SaveStructure<Vec4, Mat4> {
    SaveStructure {
        components: old
            .components
            .into_iter()
            .map(|(label, transform, x, y, z)| {
                (label, transform.fmap(convert_vec, convert_mat), x, y, z)
            })
            .collect(),
        player_data: PlayerData {
            transform: old.player_data.transform.fmap(convert_vec, convert_mat),
            heading: old.player_data.heading.fmap(convert_mat),
        },
    }
}
