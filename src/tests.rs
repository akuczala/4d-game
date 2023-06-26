#[cfg(test)]
mod tests{
    use std::{default, convert::Infallible};

    use serde::{Serialize, Deserialize, de::DeserializeOwned};
    use specs::{World, WorldExt, saveload::{SimpleMarkerAllocator, SerializeComponents}};

    use crate::{vector::{VectorTrait, Vec3, is_close, Vec2, Mat3}, geometry::shape::{buildshapes::ShapeBuilder, RefShapes}, components::{Shape, ShapeLabel, Transform}, build_level::build_lvl_1, saveload::{save_level, SaveMarker, Save, load_level}, engine::get_engine_dispatcher_builder};

    fn get_cube_label() -> ShapeLabel {
        ShapeLabel("Cube".to_string())
    }
    fn build_ref_shapes<V: VectorTrait>() -> RefShapes<V> {
        let mut ref_shapes = RefShapes::new();
        ref_shapes.insert(get_cube_label(), ShapeBuilder::build_cube(1.0).build());
        ref_shapes.insert(ShapeLabel("Coin".to_string()), ShapeBuilder::build_coin().build());
        ref_shapes
    }

    fn new_world() -> World {
        let mut world = World::new();
        world.register::<SaveMarker>();
        //world.write_resource::<SimpleMarkerAllocator<Save>>();
        world.insert::<SimpleMarkerAllocator<Save>>(SimpleMarkerAllocator::default());
        let mut dispatcher = get_engine_dispatcher_builder::<Vec3, Vec2, Mat3>().build();
        dispatcher.setup(&mut world);
        world
    }
    #[test]
    fn serialize_vec_structs() {
        fn test_json<T: Serialize +  DeserializeOwned>(t: T) -> T {
            let writer = Vec::new();
            let mut serializer = serde_json::Serializer::new(writer);
            //let serializer = 
            //let serialized = serde_json::to_string(&t).unwrap();
            t.serialize(&mut serializer);
            let serialized = String::from_utf8(serializer.into_inner()).unwrap();
            println!("serialized = {}", serialized);
            let deserialized: T = serde_json::from_str(&serialized).unwrap();
            //println!("deserialized = {:?}", deserialized);
            deserialized
        }
        fn generic<V, M>(v: V)
        where
            V: VectorTrait<M = M> + Serialize + DeserializeOwned,
            M: Serialize + DeserializeOwned
        {
            assert!(V::is_close(test_json(v), v));
            
            let shape: Shape<V> = ShapeBuilder::build_cube(1.0).build();
            test_json(shape.clone());

            
            test_json(build_ref_shapes::<V>());

            let transform: Transform<V, V::M> = Transform::identity().with_rotation(0, 1, 0.1);
            test_json(transform);

        }

        let v = Vec3::new(1.0, 2.0, 3.0);
        generic(v);

    }

    #[test]
    fn serialize_world() {
        let mut ref_shapes = build_ref_shapes::<Vec3>();
        let mut world = new_world();
        build_lvl_1(&mut world, &mut ref_shapes, &get_cube_label());
        let initial_count = world.read_component::<Shape<Vec3>>().count();
        //let mut writer = Vec::new();
        let mut serializer = serde_json::Serializer::new(Vec::new());
        
        let result = save_level::<Vec3, Mat3>(
            &world,
            &mut serializer
        );
        let serialized = match result {
                Ok(_) => String::from_utf8(serializer.into_inner()).unwrap(),
                Result::Err(_) => "Err!!!".to_string()
        };
        let mut deserialized_world = new_world();
        let mut deserializer = serde_json::Deserializer::from_str(&serialized);
        let result = load_level::<Vec3, Mat3>(&mut deserialized_world, &mut deserializer);
        if let Result::Err(_) = result {
            panic!("Boy did that go wrong");
        }
        assert_eq!(deserialized_world.read_component::<Shape<Vec3>>().count(), initial_count);
        


    }
}