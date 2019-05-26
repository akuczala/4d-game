#[macro_use]
extern crate glium;
#[allow(dead_code)]
mod vector;
#[allow(dead_code)]
mod geometry;
#[allow(dead_code)]
mod colors;
mod graphics;

fn main() {
    //test_glium_2d();
    graphics::graphics3d::test_glium_3d();
}