use std::ops::Drop;
use std::{convert::Infallible, fs};

use ron::error::SpannedResult;
use ron::ser::to_string_pretty;
use serde::{de::DeserializeOwned, Deserialize, Serialize, Serializer};
use specs::{
    saveload::{DeserializeComponents, SerializeComponents, SimpleMarker, SimpleMarkerAllocator},
    world::EntitiesRes,
    Builder, Entities, Join, LazyUpdate, Read, ReadStorage, World, WorldExt, Write, WriteStorage,
};
use transform::Transformable;

use crate::coin::Coin;
use crate::components::StaticCollider;
use crate::{
    components::{RefShapes, Shape, ShapeLabel, ShapeTexture, ShapeType, SingleFace, Transform},
    ecs_utils::Componentable,
    geometry::transform,
    shape_entity_builder::ShapeEntityBuilderV,
    vector::VectorTrait,
};

use serde_json::{de::StrRead, Serializer as JSONSerializer};
pub struct Save;
pub type SaveMarker = SimpleMarker<Save>;
pub type SaveMarkerAllocator = SimpleMarkerAllocator<Save>;

// there is some issue convincing rust that the trait bounds are ok if I try to
// 1. write save_level with a generic serializer S and
// 2. call this method with a json serializer
// so for now i have type aliases instead
type WriteBuffer = Vec<u8>;
type ComponentSerializer = serde_json::Serializer<WriteBuffer>;
type SerializerReturns = Result<(), serde_json::Error>;

type ReadBuffer<'a> = StrRead<'a>;
type ComponentDeserializer<'a> = serde_json::Deserializer<ReadBuffer<'a>>;
type DeseralizerReturns = SerializerReturns;

// TODO: practice macros?
type LevelSaveComponents<'a, V, M, U> = (
    ReadStorage<'a, ShapeLabel>,
    ReadStorage<'a, Transform<V, M>>,
    ReadStorage<'a, ShapeTexture<U>>,
    ReadStorage<'a, StaticCollider>,
    ReadStorage<'a, Coin>,
);
type LevelSaveComponentsV<'a, V> =
    LevelSaveComponents<'a, V, <V as VectorTrait>::M, <V as VectorTrait>::SubV>;

type LevelLoadComponents<'a, V, M, U> = (
    WriteStorage<'a, ShapeLabel>,
    WriteStorage<'a, Transform<V, M>>,
    WriteStorage<'a, ShapeTexture<U>>, // TODO: this saves ALL THE LINES for fuzz textures
    WriteStorage<'a, StaticCollider>,
    WriteStorage<'a, Coin>,
);

pub fn save_level<V>(world: &World, serializer: &mut ComponentSerializer) -> SerializerReturns
where
    V: VectorTrait + Componentable + Serialize + DeserializeOwned,
    V::M: Componentable + Serialize + DeserializeOwned,
    V::SubV: Componentable + Serialize + DeserializeOwned,
{
    let (save_storage, entities, markers) = world.system_data::<(
        LevelSaveComponents<V, V::M, V::SubV>,
        Entities,
        ReadStorage<SaveMarker>,
    )>();
    SerializeComponents::<Infallible, SaveMarker>::serialize(
        &save_storage,
        &entities,
        &markers,
        serializer,
    )
    // eventually add hashmap of some sort for textures
    // save ref shapes + other resources
}

pub fn load_level<V>(
    world: &mut World,
    ref_shapes: RefShapes<V>,
    deserializer: &mut ComponentDeserializer,
) -> DeseralizerReturns
where
    V: Componentable + Serialize + DeserializeOwned + VectorTrait,
    V::M: Componentable + Serialize + DeserializeOwned + Clone,
    V::SubV: Componentable + Serialize + DeserializeOwned + Clone,
{
    let result = {
        let (mut load_storage, entities, mut marker_storage, mut marker_allocator) = world
            .system_data::<(
                LevelLoadComponents<V, V::M, V::SubV>,
                Entities,
                WriteStorage<SaveMarker>,
                Write<SaveMarkerAllocator>,
            )>();
        DeserializeComponents::<Infallible, SaveMarker>::deserialize(
            &mut load_storage,
            &entities,
            &mut marker_storage,
            &mut marker_allocator,
            deserializer,
        )
    };
    world.insert(ref_shapes);
    result.map(|_| {
        //append_shape_components::<V>(world);
        world.maintain()
    })
}

type EntitySave<V, U, M> = (
    ShapeLabel,
    Transform<V, M>,
    ShapeTexture<U>,
    Option<StaticCollider>,
    Option<Coin>,
);

#[derive(Clone, Serialize, Deserialize)]
struct SaveStructure<V, U, M> {
    components: Vec<EntitySave<V, U, M>>,
}
type SaveStructureV<V> = SaveStructure<V, <V as VectorTrait>::SubV, <V as VectorTrait>::M>;

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
    SaveStructure { components }
}

fn serialize_level<V>(save_struct: &SaveStructureV<V>) -> Result<String, ron::Error>
where
    V: VectorTrait + Serialize,
    V::M: Serialize,
    V::SubV: Serialize,
{
    to_string_pretty(save_struct, Default::default())
}

fn deserialize_level<V>(level_str: &str) -> SpannedResult<SaveStructureV<V>>
where
    V: VectorTrait + DeserializeOwned,
    V::M: DeserializeOwned,
    V::SubV: DeserializeOwned,
{
    ron::from_str(level_str)
}

fn append_shape_components_new<V>(
    save_struct: SaveStructureV<V>,
    ref_shapes: &RefShapes<V>,
    world: &mut World,
) where
    V: Componentable + VectorTrait,
    V::M: Componentable,
    V::SubV: Componentable,
{
    //let (labels, transforms, entities, ref_shapes, lazy) = world.system_data::<(ReadStorage<ShapeLabel>, ReadStorage<Transform<V, V::M>>, Entities, Read<RefShapes<V>>, Read<LazyUpdate>)>();

    for (label, transform, texture, maybe_collider, maybe_coin) in save_struct.components {
        ShapeEntityBuilderV::<V>::new_from_ref_shape(ref_shapes, label)
            .with_transform(transform)
            .with_texture(texture)
            .with_collider(maybe_collider)
            .build(world)
            .maybe_with(maybe_coin)
            .build();
    }
}

pub fn save_level_to_file<V>(path: &str, world: &mut World) -> std::result::Result<(), ()>
where
    V: Componentable + VectorTrait + Serialize,
    V::M: Componentable + Serialize,
    V::SubV: Componentable + Serialize,
{
    serialize_level(&build_save_structure::<V>(world))
        .map_err(|e| println!("Could not serialize level: {}", e))
        .and_then(|s| fs::write(path, s).map_err(|e| println!("Could not save to {}: {}", path, e)))
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
    std::fs::read_to_string(path)
        .map_err(|e| println!("Error loading level {}: {}", path, e))
        .and_then(|s| {
            deserialize_level(&s).map_err(|e| println!("Could not parse level file: {}", e))
        })
        .map(|save_struct| append_shape_components_new(save_struct, ref_shapes, world))
}

fn mark_components(_world: &mut World) {
    todo!()
}

pub fn write_to_save_file() {
    todo!()
}
