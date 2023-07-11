#[cfg(test)]
mod tests {
    use std::{convert::Infallible, default};

    use serde::{de::DeserializeOwned, Deserialize, Serialize};
    use specs::{
        saveload::{SerializeComponents, SimpleMarkerAllocator},
        World, WorldExt,
    };

    use crate::{
        build_level::{build_lvl_1, build_shape_library},
        components::{Shape, ShapeLabel, Transform},
        config::{self, save_config, Config},
        constants::CUBE_LABEL_STR,
        engine::get_engine_dispatcher_builder,
        geometry::shape::{buildshapes::ShapeBuilder, RefShapes},
        saveload::{load_level, save_level, Save, SaveMarker},
        vector::{is_close, Mat3, Vec2, Vec3, VectorTrait},
    };

    fn new_world() -> World {
        let mut world = World::new();
        world.register::<SaveMarker>();
        //world.write_resource::<SimpleMarkerAllocator<Save>>();
        world.insert::<SimpleMarkerAllocator<Save>>(SimpleMarkerAllocator::default());
        let mut dispatcher = get_engine_dispatcher_builder::<Vec3>().build();
        dispatcher.setup(&mut world);
        world
    }
    #[test]
    fn serialize_vec_structs() {
        fn test_json<T: Serialize + DeserializeOwned>(t: T) -> T {
            let writer = Vec::new();
            let mut serializer = serde_json::Serializer::new(writer);
            //let serializer =
            //let serialized = serde_json::to_string(&t).unwrap();
            t.serialize(&mut serializer).unwrap();
            let serialized = String::from_utf8(serializer.into_inner()).unwrap();
            println!("serialized = {}", serialized);
            let deserialized: T = serde_json::from_str(&serialized).unwrap();
            //println!("deserialized = {:?}", deserialized);
            deserialized
        }
        fn generic<V, M>(v: V)
        where
            V: VectorTrait<M = M> + Serialize + DeserializeOwned,
            M: Serialize + DeserializeOwned,
        {
            assert!(V::is_close(test_json(v), v));

            let shape: Shape<V> = ShapeBuilder::build_cube(1.0).build();
            test_json(shape.clone());

            test_json(build_shape_library::<V>());

            let transform: Transform<V, V::M> = Transform::identity().with_rotation(0, 1, 0.1);
            test_json(transform);
        }

        let v = Vec3::new(1.0, 2.0, 3.0);
        generic(v);
    }

    #[test]
    fn serialize_world() {
        let mut ref_shapes = build_shape_library::<Vec3>();
        let mut world = new_world();
        build_lvl_1(&mut world, &mut ref_shapes);
        let initial_count = world.read_component::<Shape<Vec3>>().count();
        //let mut writer = Vec::new();
        let mut serializer = serde_json::Serializer::new(Vec::new());

        let result = save_level::<Vec3>(&world, &mut serializer);
        let serialized = match result {
            Ok(_) => String::from_utf8(serializer.into_inner()).unwrap(),
            Result::Err(_) => "Err!!!".to_string(),
        };
        let mut deserialized_world = new_world();
        let mut deserializer = serde_json::Deserializer::from_str(&serialized);
        let result = load_level::<Vec3>(&mut deserialized_world, &mut deserializer);
        if let Result::Err(_) = result {
            panic!("Boy did that go wrong");
        }
        assert_eq!(
            deserialized_world.read_component::<Shape<Vec3>>().count(),
            initial_count
        );
    }

    #[test]
    fn load_config() {
        println!("{:?}", config::load_config())
    }
    #[test]
    fn test_save_config() {
        save_config(Config::default()).unwrap()
    }
}
