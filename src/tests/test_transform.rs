use crate::{
    geometry::{affine_transform::AffineTransform, transform::Scaling},
    tests::random_vec,
    vector::{MatrixTrait, Vec4, VectorTrait},
};

use super::random_transform;

#[test]
fn test_inverse() {
    type V = Vec4;
    let transform = random_transform::<V>();
    let v = random_vec::<V>();
    let inverse_transform = transform.inverse();
    assert!(V::is_close(
        transform.transform_vec(&inverse_transform.transform_vec(&v)),
        v
    ));
    assert!(V::is_close(
        inverse_transform.transform_vec(&transform.transform_vec(&v)),
        v
    ));
}

#[test]
fn test_transform_to_affine() {
    type V = Vec4;
    let transform = random_transform::<V>();
    let v = random_vec::<V>();
    assert!(V::is_close(
        transform.transform_vec(&v),
        AffineTransform::from(transform).transform_vec(&v)
    ));
}

#[test]
fn test_decompose() {
    use crate::vector::{Mat4, Vec4};
    let s = Scaling::Vector(Vec4::new(2.0, 3.0, 5.0, 7.0));
    let rot_mat = Mat4::from_arr(&[
        [-0.692_144_1, 0.447_720_9, 0.55884119, -0.09043814],
        [-0.19507629, -0.72900476, 0.2438655, -0.609_119_8],
        [0.34303542, -0.2792939, 0.73238738, 0.517_619_9],
        [0.6043248, 0.43599655, 0.30304269, -0.594_023_3],
    ]);
    let transform = AffineTransform::new(Some(Vec4::zero()), Some(rot_mat.dot(s.get_mat())));
    let (rot_mat_recon, s_recon) = transform.decompose_rotation_scaling();
    println!("{}", transform.frame.transpose());
    println!(
        "{}",
        match s_recon {
            Scaling::Vector(v) => v,
            Scaling::Scalar(_f) => Vec4::zero(),
        }
    );
    println!("{}", rot_mat_recon)
}
