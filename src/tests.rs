#[cfg(test)]
mod tests{
    use serde::{Serialize, Deserialize, de::DeserializeOwned};

    use crate::{vector::{VectorTrait, Vec3, is_close}, geometry::shape::buildshapes::ShapeBuilder, components::Shape};

    #[test]
    fn serialize_vec() {
        fn test_json<T: Serialize + DeserializeOwned>(t: T) -> T {
            let serialized = serde_json::to_string(&t).unwrap();
            println!("serialized = {}", serialized);
            let deserialized: T = serde_json::from_str(&serialized).unwrap();
            //println!("deserialized = {:?}", deserialized);
            deserialized
        }
        fn generic<V: VectorTrait + Serialize + DeserializeOwned>(v: V) {
            assert!(V::is_close(test_json(v), v));
            
            let shape: Shape<V> = ShapeBuilder::build_cube(1.0).build();
            test_json(shape);

        }

        let v = Vec3::new(1.0, 2.0, 3.0);
        generic(v);
        // let serialized = serde_json::to_string(&v).unwrap();
        // println!("serialized = {}", serialized);

        // let deserialized: Vec3 = serde_json::from_str(&serialized).unwrap();
        // println!("deserialized = {:?}", deserialized);
        // assert!(Vec3::is_close(deserialized, v))
    }
}