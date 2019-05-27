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
        color = vec4(my_attr,0.0,1.0);
    }
    "#;

pub fn test_glium() {
    use crate::vector::{Vec2};
    let (mut events_loop, display) = init_glium();


    let shape = vec![Vec2::new(-0.5, -0.5),Vec2::new(-0.0, 0.5),Vec2::new( 0.5, -0.25)];
    let shape : Vec<Vertex> = shape.iter().map(|v| Vertex{position : *v.get_arr() as [f32 ; 2]}).collect();

    let vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);


    let program = glium::Program::from_source(&display,
        VERTEX_SHADER_SRC,
        FRAGMENT_SHADER_SRC, None)
    .unwrap();

    let mut t: f32 = -0.5;
    let mut closed = false;
    while !closed {
        // we update `t`
        t += 0.0002;
        if t > 0.5 {
            t = -0.5;
        }
        let mut target = display.draw();

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
                [ t , 0.0, 0.0, 1.0f32],
            ]
        };

        target.clear_color(0.0,0.0,1.0,1.0);
        target.draw(&vertex_buffer, &indices, &program, &uniforms,
            &Default::default()).unwrap();
        target.finish().unwrap();
        
        listen_events(&mut events_loop, &mut closed)
    }
}

pub fn test_glium_2() {
    use crate::vector::{Vec2,Vec3};
    use crate::draw::Camera;
    let (mut events_loop, display) = init_glium();

    let cylinder = crate::geometry::buildshapes::build_cylinder(2.0,2.0,32);
    let camera = crate::draw::Camera{
        pos : Vec3::new(1.0, 1.0, -10.0),
        frame : <Vec3 as VectorTrait>::M::id()
    };
    let (verts, vertis) = crate::draw::draw_wireframe(&display,&camera,cylinder);
    let shape : Vec<Vertex> = verts.iter().map(|v| Vertex{position : *v.get_arr() as [f32 ; 2]}).collect();
    let convert_vertis : Vec<u16> = vertis.iter().map(|v| *v as u16).collect();
    let vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();
    let indices = glium::IndexBuffer::new(
        &display,
        glium::index::PrimitiveType::LinesList
        ,&convert_vertis
        )
    .unwrap();

    let program = glium::Program::from_source(&display,
        VERTEX_SHADER_SRC,
        FRAGMENT_SHADER_SRC, None)
    .unwrap();

    let mut t: f32 = -0.5;
    let mut closed = false;
    while !closed {
        // we update `t`
        t += 0.0002;
        if t > 0.5 {
            t = -0.5;
        }
        let mut target = display.draw();

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
                [ t , 0.0, 0.0, 1.0f32],
            ]
        };

        target.clear_color(0.0,0.0,1.0,1.0);
        target.draw(&vertex_buffer, &indices, &program, &uniforms,
            &Default::default()).unwrap();
        target.finish().unwrap();
        
        listen_events(&mut events_loop, &mut closed)
    }
}