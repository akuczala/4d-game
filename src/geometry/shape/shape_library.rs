use std::{
    collections::HashMap,
    fmt::{self, Display},
};

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use specs::{Component, VecStorage};

use crate::{
    constants::{
        COIN_LABEL_STR, CUBE_LABEL_STR, INVERTED_CUBE_LABEL_STR, INVERTED_PIPE_LABEL_STR,
        ONE_SIDED_FACE_LABEL_STR, OPEN_CUBE_LABEL_STR, OPEN_INVERTED_CUBE_LABEL_STR,
        TWO_SIDED_FACE_LABEL_STR,
    },
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
impl ShapeLabel {
    pub fn from_str(str: &str) -> Self {
        ShapeLabel(str.to_string())
    }
}

#[derive(Serialize, Deserialize)]
pub struct RefShapes<V>(HashMap<ShapeLabel, Shape<V>>);
impl<V: VectorTrait> RefShapes<V> {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn get(&self, key: &ShapeLabel) -> Option<&Shape<V>> {
        self.0.get(key)
    }
    pub fn get_unwrap(&self, key: &ShapeLabel) -> &Shape<V> {
        self.get(key)
            .unwrap_or_else(|| panic!("Ref shape {} not found", key))
    }
    pub fn insert(&mut self, key: ShapeLabel, value: Shape<V>) -> Option<Shape<V>> {
        self.0.insert(key, value)
    }
    pub fn get_labels(&self) -> Vec<ShapeLabel> {
        self.0.keys().cloned().collect_vec()
    }
}
impl<V: VectorTrait> Default for RefShapes<V> {
    fn default() -> Self {
        Self(HashMap::new())
    }
}

pub fn build_shape_library<V: VectorTrait>() -> RefShapes<V> {
    let mut ref_shapes: RefShapes<V> = RefShapes::new();
    let cube = ShapeBuilder::<V>::build_cube(1.0).build();
    let sub_cube = ShapeBuilder::<V::SubV>::build_cube(1.0).build();
    let inverted_cube = invert_normals(&cube);
    let open_cube = remove_face(cube.clone(), cube.faces.len() - 1);
    let open_inverted_cube = remove_face(inverted_cube.clone(), inverted_cube.faces.len() - 1);
    let inverted_pipe = make_pipe(V::one_hot(-1), inverted_cube.clone());

    let coin: Shape<V> = ShapeBuilder::<V>::build_coin().build();
    ref_shapes.insert(ShapeLabel::from_str(CUBE_LABEL_STR), cube);
    ref_shapes.insert(ShapeLabel::from_str(COIN_LABEL_STR), coin);
    ref_shapes.insert(
        ShapeLabel::from_str(ONE_SIDED_FACE_LABEL_STR),
        convex_shape_to_face_shape(sub_cube.clone(), false),
    );
    ref_shapes.insert(
        ShapeLabel::from_str(TWO_SIDED_FACE_LABEL_STR),
        convex_shape_to_face_shape(sub_cube, true),
    );
    ref_shapes.insert(
        ShapeLabel::from_str(INVERTED_CUBE_LABEL_STR),
        inverted_cube.to_generic(),
    );
    ref_shapes.insert(ShapeLabel::from_str(OPEN_CUBE_LABEL_STR), open_cube);
    ref_shapes.insert(
        ShapeLabel::from_str(OPEN_INVERTED_CUBE_LABEL_STR),
        open_inverted_cube,
    );
    ref_shapes.insert(ShapeLabel::from_str(INVERTED_PIPE_LABEL_STR), inverted_pipe);
    ref_shapes.insert(
        ShapeLabel::from_str("TestPrism"),
        ShapeBuilder::build_prism(V::DIM, &[2.0, 1.0], &[3, 4]).build(),
    );
    ref_shapes
}
