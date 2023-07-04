use std::convert::Infallible;

use serde::{de::DeserializeOwned, Serialize, Serializer};
use specs::{
    saveload::{DeserializeComponents, SerializeComponents, SimpleMarker, SimpleMarkerAllocator},
    world::EntitiesRes,
    Entities, ReadStorage, World, WorldExt, Write, WriteStorage,
};

use crate::{
    components::{Shape, ShapeLabel, ShapeType, SingleFace, Transform},
    ecs_utils::Componentable,
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
type LevelSaveComponents<'a, V, M> = (
    ReadStorage<'a, Shape<V>>,
    ReadStorage<'a, ShapeLabel>,
    ReadStorage<'a, Transform<V, M>>, //ReadStorage<'a, ShapeType<V>>,
                                      //ReadStorage<'a, SingleFace<V>>,
);

type LevelLoadComponents<'a, V, M> = (
    WriteStorage<'a, Shape<V>>,
    WriteStorage<'a, ShapeLabel>,
    WriteStorage<'a, Transform<V, M>>, //WriteStorage<'a, ShapeType<V>>,
                                       //WriteStorage<'a, SingleFace<V>>,
);

pub fn save_level<V>(world: &World, serializer: &mut ComponentSerializer) -> SerializerReturns
where
    V: VectorTrait + Componentable + Serialize + DeserializeOwned,
    V::M: Componentable + Serialize + DeserializeOwned,
{
    let (save_storage, entities, markers) = world.system_data::<(
        LevelSaveComponents<V, V::M>,
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
    deserializer: &mut ComponentDeserializer,
) -> DeseralizerReturns
where
    V: Componentable + Serialize + DeserializeOwned + VectorTrait,
    V::M: Componentable + Serialize + DeserializeOwned + Clone,
{
    let (mut load_storage, entities, mut marker_storage, mut marker_allocator) = world
        .system_data::<(
            LevelLoadComponents<V, V::M>,
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
}

fn mark_components(_world: &mut World) {
    todo!()
}

pub fn write_to_save_file() {
    todo!()
}
