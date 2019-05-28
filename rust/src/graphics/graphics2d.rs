use glium::Surface;
use super::init_glium;
use super::listen_events;

use crate::vector::{VectorTrait,MatrixTrait};

#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 2],
}
implement_vertex!(Vertex, position);

pub const VERTEX_SHADER_SRC : &str = r#"
    #version 140

    in vec2 position;
    out vec2 my_attr;

    uniform mat4 perspective;
    uniform mat4 matrix;

    void main() {
        my_attr = position;
        gl_Position = perspective * matrix * vec4(position, 0.0, 1.0);
    }
    "#;
pub const FRAGMENT_SHADER_SRC : &str = r#"
    #version 140

    in vec2 my_attr;
    out vec4 color;

    void main() {
        color = vec4(1.0,1.0,1.0,1.0);
    }
    "#;
use crate::vector::{Vec2};
fn verts_to_vertexes(verts : &Vec<Vec2>) -> Vec<Vertex> {
    verts.iter().map(|v| Vertex{position : *v.get_arr() as [f32 ; 2]}).collect()
}
pub fn test_glium_2() {
    use crate::vector::{Vec2,Vec3};
    use crate::draw::Camera;
    use super::ButtonsPressed;
    let mut pressed = ButtonsPressed::new();

    let (mut events_loop, display) = init_glium();

    let mut cylinder = crate::geometry::buildshapes::build_cylinder(1.0,2.0,32);
    let camera = Camera{
        pos : Vec3::new(0.0, 0.0, -10.0),
        frame : <Vec3 as VectorTrait>::M::id()
    };
    

    let program = glium::Program::from_source(&display,
        VERTEX_SHADER_SRC,
        FRAGMENT_SHADER_SRC, None)
    .unwrap();

    let mut t: f32 = -0.5;
    let mut closed = false;
    let mut pos = Vec3::zero();
    let dz = 0.001;
    let mut update = true;
    while !closed {
        // we update `t`
        t += 0.02;
        if t > 5.5 {
            t = -5.5;
        }

        let perspective = [
            [1., 0., 0., 0.],
            [0., 1., 0., 0.],
            [0., 0., 1., 0.],
            [0., 0., 0., 1.032f32]];
        let uniforms = uniform! {
            perspective : perspective,
            matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [ 0.0, 0.0, 0.0 , 1.0f32],
            ]
        };
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
        if update {
            cylinder.set_pos(&pos);
            cylinder.rotate(1,2,0.01f32);
            let (verts, vertis) = crate::draw::draw_wireframe(&display,&camera,&cylinder);

            let vertexes : Vec<Vertex> = verts_to_vertexes(&verts);
            let convert_vertis : Vec<u16> = vertis.iter().map(|v| *v as u16).collect();
            //would like to use VertexBuffer::dynamic and modify buffer rather than recreate over and over
            let vertex_buffer = glium::VertexBuffer::new(&display, &vertexes).unwrap();
            let indices = glium::IndexBuffer::new(
                &display,
                glium::index::PrimitiveType::LinesList
                ,&convert_vertis
                )
            .unwrap();

            let mut target = display.draw();
            target.clear_color(0.0,0.1,0.1,1.0);
            target.draw(&vertex_buffer, &indices, &program, &uniforms,
                &Default::default()).unwrap();
            target.finish().unwrap();

            update = false;
        }
        
        listen_events(&mut events_loop, &mut closed, &mut pressed);

        
    }
}