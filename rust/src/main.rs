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
//mod text_wrapper;

fn main() {
    test_glium_2();
    //graphics::graphics3d::test_glium_3d();
}
use crate::vector::{VectorTrait,MatrixTrait};
use crate::graphics::Graphics;
use crate::graphics::listen_events;

pub fn test_glium_2() {
    use crate::vector::{Vec3};
    use crate::draw::Camera;
    use crate::graphics::ButtonsPressed;
    let mut pressed = ButtonsPressed::new();

    let mut graphics =  crate::graphics::Graphics2d::new();

    let mut cylinder = crate::geometry::buildshapes::build_cylinder(1.0,2.0,32);
    let camera = Camera{
        pos : Vec3::new(0.0, 0.0, -10.0),
        frame : <Vec3 as VectorTrait>::M::id()
    };
    

    let (verts, vertis) = crate::draw::draw_wireframe(&camera,&cylinder);
    //vertex buffer (and presumably index buffer) do not allow size of array
    //to change (at least using the write operation)
    graphics.new_vertex_buffer(&verts);
    graphics.new_index_buffer(&vertis);

    //let mut t: f32 = -0.5;
    let mut closed = false;
    let mut pos = Vec3::zero();
    let dz = 0.01;
    let mut update = true;
    while !closed {

        if pressed.w {
            pos = pos + Vec3::new(0.0,0.0,dz);
            update = true;
        }
        if pressed.s {
            pos = pos - Vec3::new(0.0,0.0,dz);
            update = true;
        }
        if pressed.d {
            pos = pos + Vec3::new(dz,0.0,0.0);
            update = true;
        }
        if pressed.a {
            pos = pos - Vec3::new(dz,0.0,0.0);
            update = true;
        }
        if !pressed.space {
            println!("{}",cylinder.get_pos());
            pressed.space = true;
        }
        if update {
            cylinder.set_pos(&pos);
            cylinder.rotate(1,2,0.001f32);
            let (verts, vertis) = crate::draw::draw_wireframe(&camera,&cylinder);
            graphics.draw_lines(&verts,&vertis);

            
            update = false;
        }
        
        listen_events(&mut graphics.get_event_loop(), &mut closed, &mut pressed);

        
    }
}
