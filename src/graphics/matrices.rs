use std::f32::consts::PI;

use crate::vector::VecIndex;

pub type Matrix4 = [[f32; 4]; 4];

pub const IDENTITY_MATRIX: Matrix4 = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0f32],
];

pub fn build_view_matrix(dim: VecIndex) -> Matrix4 {
    match dim {
        2 => IDENTITY_MATRIX,
        3 => build_view_matrix_3d(&[3.0, 3.0, -4.0], &[-3.0, -3.0, 4.0], &[0.0, 1.0, 0.0]),
        //3 => build_view_matrix_3d(&[4.0, 2.0, 2.0], &[-2.0, -1.0, -1.0], &[0.0, 1.0, 0.0]),
        _ => panic!("Invalid dimension"),
    }
}

// TODO: rename
pub fn build_other_view_matrix(dim: VecIndex) -> Matrix4 {
    match dim {
        2 => IDENTITY_MATRIX,
        //3 => build_view_matrix_3d(&[2.0, 2.0, -4.0], &[-1.0, -1.0, 2.0], &[0.0, 1.0, 0.0]),
        3 => build_view_matrix_3d(&[4.0, 3.0, 3.0], &[-4.0, -3.0, -3.0], &[0.0, 1.0, 0.0]),
        _ => panic!("Invalid dimension"),
    }
}

pub fn build_perspective_matrix(dim: VecIndex, width: u32, height: u32) -> Matrix4 {
    match dim {
        2 => build_perspective_mat_2d(width, height),
        3 => build_perspective_mat_3d(width, height),
        _ => panic!("Invalid dimension"),
    }
}

fn build_view_matrix_3d(position: &[f32; 3], direction: &[f32; 3], up: &[f32; 3]) -> Matrix4 {
    let f = {
        let f = direction;
        let len = f[0] * f[0] + f[1] * f[1] + f[2] * f[2];
        let len = len.sqrt();
        [f[0] / len, f[1] / len, f[2] / len]
    };

    let s = [
        up[1] * f[2] - up[2] * f[1],
        up[2] * f[0] - up[0] * f[2],
        up[0] * f[1] - up[1] * f[0],
    ];

    let s_norm = {
        let len = s[0] * s[0] + s[1] * s[1] + s[2] * s[2];
        let len = len.sqrt();
        [s[0] / len, s[1] / len, s[2] / len]
    };

    let u = [
        f[1] * s_norm[2] - f[2] * s_norm[1],
        f[2] * s_norm[0] - f[0] * s_norm[2],
        f[0] * s_norm[1] - f[1] * s_norm[0],
    ];

    let p = [
        -position[0] * s_norm[0] - position[1] * s_norm[1] - position[2] * s_norm[2],
        -position[0] * u[0] - position[1] * u[1] - position[2] * u[2],
        -position[0] * f[0] - position[1] * f[1] - position[2] * f[2],
    ];

    [
        [s_norm[0], u[0], f[0], 0.0],
        [s_norm[1], u[1], f[1], 0.0],
        [s_norm[2], u[2], f[2], 0.0],
        [p[0], p[1], p[2], 1.0],
    ]
}

fn build_perspective_mat_2d(width: u32, height: u32) -> Matrix4 {
    let aspect_ratio = height as f32 / width as f32;
    let fov: f32 = PI / 3.0;
    //let zfar = 1024.0;
    //let znear = 0.1;

    let f = 1.0 / (fov / 2.0).tan();

    [
        [f * aspect_ratio, 0., 0., 0.],
        [0., f, 0., 0.],
        [0., 0., 1., 0.],
        [0., 0., 0., 1.032f32],
    ]
}

pub fn build_perspective_mat_3d(width: u32, height: u32) -> Matrix4 {
    let aspect_ratio = height as f32 / width as f32;
    //let fov: f32 = 3.141592 / 3.0; //nearly fish eye
    //let fov : f32 = 3.141592 / 8.0;
    let fov: f32 = PI / 16.0; //comparable to 3d
                              //let zfar = 1024.0;/
    let zfar = 100.0;
    let znear = 0.1;

    let f = 1.0 / (fov / 2.0).tan();
    [
        [f * aspect_ratio, 0., 0., 0.],
        [0., f, 0., 0.],
        [0., 0., (zfar + znear) / (zfar - znear), 1.0],
        [0., 0., 0., 1.032f32],
    ]
}
