use std::convert::Infallible;

use serde::{Serialize, de::DeserializeOwned, Serializer};
use specs::{World, saveload::{SimpleMarker, SerializeComponents, DeserializeComponents, SimpleMarkerAllocator}, WorldExt, Entities, WriteStorage, Write};

use crate::{vector::VectorTrait, ecs_utils::Componentable, components::Shape};

use serde_json::{Serializer as JSONSerializer, de::StrRead};
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

pub fn save_level<V>(world: &World, serializer: &mut ComponentSerializer) -> SerializerReturns
where
    V: Componentable + Serialize + DeserializeOwned + Clone
{
    SerializeComponents::<Infallible, SaveMarker>::serialize(
        &(world.read_component::<Shape<V>>(),),
        &world.entities(),
        &world.read_component::<SaveMarker>(),
        serializer
    )
    // save components  
    // eventually add hashmap of some sort for textures
    // save ref shapes + other resources
}

pub fn load_level<V>(world: &mut World, deserializer: &mut ComponentDeserializer) -> DeseralizerReturns
where
    V: Componentable + Serialize + DeserializeOwned + Clone
{
    //let mut shape_storage = world.write_component::<Shape<V>>();
    let (
        entities,
        mut marker_storage,
        mut marker_allocator,
        shape_storage,
    ) = world.system_data::<(
        Entities,
        WriteStorage<SaveMarker>,
        Write<SaveMarkerAllocator>,
        WriteStorage<Shape<V>>,
    )>();
    DeserializeComponents::<Infallible, SaveMarker>::deserialize(
        &mut (shape_storage,),
        &entities,
        &mut marker_storage,
        &mut marker_allocator,
        deserializer
    )
}

fn mark_components(world: &mut World) {
    todo!()
}