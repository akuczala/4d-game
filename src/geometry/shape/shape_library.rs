use std::fmt::{self, Display};

use serde::{Deserialize, Serialize};
use specs::{Component, VecStorage};

use crate::{
    constants::{
        COIN_LABEL_STR, CUBE_LABEL_STR, INVERTED_CUBE_LABEL_STR, INVERTED_PIPE_LABEL_STR,
        ONE_SIDED_FACE_LABEL_STR, OPEN_CUBE_LABEL_STR, OPEN_INVERTED_CUBE_LABEL_STR,
        TWO_SIDED_FACE_LABEL_STR,
    },
    utils::ResourceLibrary,
    vector::VectorTrait,
};

use super::{
    buildshapes::{
        convex_shape_to_face_shape, invert_normals, make_pipe, remove_face, ShapeBuilder,
    },
    Shape,
};

// TODO: consider merging with Shape
// might be a bad idea - could contain in larger struct?
#[derive(Component, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
#[storage(VecStorage)]
pub struct ShapeLabel(pub String);
impl Display for ShapeLabel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl From<&str> for ShapeLabel {
    fn from(value: &str) -> Self {
        ShapeLabel(value.to_string())
    }
}
impl From<String> for ShapeLabel {
    fn from(value: String) -> Self {
        ShapeLabel(value)
    }
}

pub type RefShapes<V> = ResourceLibrary<ShapeLabel, Shape<V>>;

pub fn build_shape_library<V: VectorTrait>() -> RefShapes<V> {
    let cube = ShapeBuilder::<V>::build_cube(1.0).build();
    let sub_cube = ShapeBuilder::<V::SubV>::build_cube(1.0).build();
    let inverted_cube = invert_normals(&cube);
    let open_cube = remove_face(cube.clone(), cube.faces.len() - 1);

    RefShapes::build(vec![
        (CUBE_LABEL_STR.into(), cube.clone()),
        (
            COIN_LABEL_STR.into(),
            ShapeBuilder::<V>::build_coin().build(),
        ),
        (
            ONE_SIDED_FACE_LABEL_STR.into(),
            convex_shape_to_face_shape(sub_cube.clone(), false),
        ),
        (
            TWO_SIDED_FACE_LABEL_STR.into(),
            convex_shape_to_face_shape(sub_cube, true),
        ),
        (
            INVERTED_CUBE_LABEL_STR.into(),
            invert_normals(&cube).to_generic(),
        ),
        (OPEN_CUBE_LABEL_STR.into(), open_cube.clone()),
        (
            OPEN_INVERTED_CUBE_LABEL_STR.into(),
            invert_normals(&open_cube),
        ),
        (
            INVERTED_PIPE_LABEL_STR.into(),
            make_pipe(V::one_hot(-1), inverted_cube.clone()),
        ),
        (
            "TestPrism".into(),
            ShapeBuilder::build_prism(V::DIM, &[2.0, 1.0], &[3, 4]).build(),
        ),
    ])
}
