#[macro_use]
extern crate glium;
#[allow(dead_code)]
mod vector;
#[allow(dead_code)]
mod geometry;
#[allow(dead_code)]
mod draw;
#[allow(dead_code)]
mod colors;
mod graphics;


fn main() {
    graphics::graphics2d::test_glium_2();
    //graphics::graphics3d::test_glium_3d();
}