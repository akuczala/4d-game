use crate::{
    constants::{
        COIN_LABEL_STR, CUBE_LABEL_STR, INVERTED_CUBE_LABEL_STR, INVERTED_PIPE_LABEL_STR,
        ONE_SIDED_FACE_LABEL_STR, OPEN_CUBE_LABEL_STR, OPEN_INVERTED_CUBE_LABEL_STR,
        TWO_SIDED_FACE_LABEL_STR,
    },
    utils::{ResourceLabel, ResourceLibrary},
    vector::VectorTrait,
};

use super::{
    buildshapes::{
        convex_shape_to_face_shape, invert_normals, make_pipe, remove_face, ShapeBuilder,
    },
    Shape,
};

pub type ShapeLabel = ResourceLabel<Shape<()>>;

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
            make_pipe(V::one_hot(-1), inverted_cube),
        ),
        (
            "TestPrism".into(),
            ShapeBuilder::build_prism(V::DIM, &[2.0, 1.0], &[3, 4]).build(),
        ),
    ])
}
