use std::iter;
/// This module is a mess of ad-hoc helper functions for converting save files between refactors.
/// Most are specific to particular changes, but can probably be generalized
use std::path::Path;

use serde::{Deserialize, Serialize};
use transform::Transform;

use crate::coin::Coin;
use crate::components::{ShapeLabel, StaticCollider};

use crate::constants::COIN_TEXTURE_LABEL_STR;
use crate::draw::texture::texture_builder::TexturePrim;
use crate::draw::texture::{FrameTextureMapping, ShapeTextureBuilder};

use crate::graphics::colors::Color;
use crate::saveload::{load_save_struct_from_file, save_save_struct_to_file};
use crate::{geometry::transform, vector::VectorTrait};

#[derive(Clone, Serialize, Deserialize)]
enum OldTextureBuilderStep {
    WithColor(Color),
    WithTexture(TexturePrim),
    MergedWith(Vec<OldTextureBuilderStep>),
    ColorByNormal,
}
#[derive(Clone, Serialize, Deserialize)]
struct OldTextureBuilder {
    steps: Vec<OldTextureBuilderStep>,
}
#[derive(Clone, Serialize, Deserialize)]
struct OldShapeTextureBuilder {
    face_textures: Vec<OldFaceTextureBuilder>,
}
impl OldShapeTextureBuilder {
    fn to_new(is_coin: bool) -> ShapeTextureBuilder {
        if is_coin {
            ShapeTextureBuilder::from_resource(COIN_TEXTURE_LABEL_STR.into())
        } else {
            ShapeTextureBuilder::default()
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
struct OldFaceTextureBuilder {
    texture: OldTextureBuilder,
    texture_mapping: Option<FrameTextureMapping>,
}

#[derive(Clone, Serialize, Deserialize)]
struct OldShapeLabel(String);
impl From<OldShapeLabel> for ShapeLabel {
    fn from(value: OldShapeLabel) -> Self {
        value.0.into()
    }
}

type OldEntitySave<V, M> = (
    OldShapeLabel,
    Transform<V, M>,
    OldShapeTextureBuilder,
    Option<StaticCollider>,
    Option<Coin>,
);
fn convert_entity_save<V, M>(
    (label, transform, _builder, collider, coin): OldEntitySave<V, M>,
) -> EntitySave<V, M> {
    (
        label.into(),
        transform,
        OldShapeTextureBuilder::to_new(coin.is_some()),
        collider,
        coin,
    )
}
type OldSaveStructure<V, M> = SaveStructureGeneric<OldEntitySave<V, M>, PlayerData<V, M>>;
type OldSaveStructureV<V> = OldSaveStructure<V, <V as VectorTrait>::M>;
fn convert_save_struct<V: VectorTrait>(old: OldSaveStructureV<V>) -> SaveStructureV<V> {
    SaveStructureV {
        components: old
            .components
            .into_iter()
            .map(convert_entity_save)
            .collect(),
        player_data: old.player_data,
    }
}

/// Used to upgrade old texture format to new
#[allow(dead_code)]
fn upgrade_old_save_file() {
    let old_save_struct: OldSaveStructureV<Vec3> =
        load_save_struct_from_file("./resources/levels/level_2.3d.ron").unwrap();
    let new_save_struct = convert_save_struct(old_save_struct);
    save_save_struct_to_file(
        Path::new("./resources/levels/level_2.3d.ron"),
        &new_save_struct,
    )
    .unwrap();
}
fn unproject_matrix<V: VectorTrait>(matrix: <V::SubV as VectorTrait>::M) -> V::M {
    matrix
        .get_rows()
        .into_iter()
        .map(V::unproject)
        .chain(iter::once(V::one_hot(-1)))
        .collect()
}
// Map 3d levels into 4d?
// this doesn't quite work because the ref shapes for 3d and 4d don't quite match.
// e.g. removing the -1th face removes different faces
//#[test]
#[allow(dead_code)]
fn unproject_level() {
    let save_struct_3d = load_save_struct_from_file("./resources/levels/level_0.3d.ron").unwrap();
    save_save_struct_to_file(
        Path::new("./resources/levels/level_0_test.4d.ron"),
        &map_save_struct(save_struct_3d, Vec4::unproject, unproject_matrix::<Vec4>),
    )
    .unwrap();
}

// I used the below code to update the rep of Vec4 from a record object to a singleton object
// may or may not be useful in the future, but I anticipate making future changes to the data representation
// this could be used in tandem with creating invariant structs for each datatype for saving, and converting
// the runtime types (which may change over time) to the invariant types
use crate::vector::{Field, Mat4, MatrixTrait, Vec3, Vec4};

use super::{EntitySave, PlayerData, SaveStructure, SaveStructureGeneric, SaveStructureV};

#[derive(Deserialize, Clone, Copy)]
struct OldVec4 {
    arr: [Field; 4],
}

#[derive(Deserialize, Clone, Copy)]
struct OldMat4(OldVec4, OldVec4, OldVec4, OldVec4);

#[allow(dead_code)]
fn upgrade_savefile() {
    let old_save_struct = load_save_struct_from_file("./old.4d.ron").unwrap();
    save_save_struct_to_file(
        Path::new("./new.4d.ron"),
        &map_save_struct(old_save_struct, convert_vec, convert_mat),
    )
    .unwrap();
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

fn map_save_struct<V, U, M, N>(
    save_struct: SaveStructure<V, M>,
    f: fn(V) -> U,
    g: fn(M) -> N,
) -> SaveStructure<U, N> {
    SaveStructure {
        components: save_struct
            .components
            .into_iter()
            .map(|(label, transform, x, y, z)| (label, transform.fmap(f, g), x, y, z))
            .collect(),
        player_data: PlayerData {
            transform: save_struct.player_data.transform.fmap(f, g),
            heading: save_struct.player_data.heading.fmap(g),
        },
    }
}
