#[cfg(test)]
mod tests{
    use crate::vector::{VectorTrait, Vec3, is_close};

    #[test]
    fn serialize_vec() {

        let v = Vec3::new(1.0, 2.0, 3.0);
        let serialized = serde_json::to_string(&v).unwrap();
        println!("serialized = {}", serialized);

        let deserialized: Vec3 = serde_json::from_str(&serialized).unwrap();
        println!("deserialized = {:?}", deserialized);
        assert!(Vec3::is_close(deserialized, v))
    }
}