use std::fs;
use std::path::Path;

use ron::error::SpannedResult;
use ron::ser::to_string_pretty;
use serde::{de::DeserializeOwned, Deserialize, Serialize, Serializer};
use specs::prelude::*;
use transform::{Transform, Transformable};

use crate::coin::Coin;
use crate::components::{RefShapes, ShapeLabel, ShapeTexture, StaticCollider};
use crate::draw::texture::texture_builder::TextureBuilder;
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
struct SaveStructure<V, M> {
    components: Vec<EntitySave<V, M>>,
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

fn append_shape_components<V>(
    save_struct: SaveStructureV<V>,
    ref_shapes: &RefShapes<V>,
    world: &mut World,
) where
    V: Componentable + VectorTrait,
    V::M: Componentable,
    V::SubV: Componentable,
{
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

pub fn save_level_to_file<V>(path: &Path, world: &mut World) -> std::result::Result<(), ()>
where
    V: Componentable + VectorTrait + Serialize,
    V::M: Componentable + Serialize,
    V::SubV: Componentable + Serialize,
{
    serialize_level(&build_save_structure::<V>(world))
        .map_err(|e| println!("Could not serialize level: {}", e))
        .and_then(|s| {
            fs::write(path, s)
                .map_err(|e| println!("Could not save to {}: {}", path.to_str().unwrap(), e))
        })
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
        .map(|save_struct| append_shape_components(save_struct, ref_shapes, world))
}
