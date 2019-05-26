use glium::Surface;
use super::init_glium;
use super::listen_events;
#[derive(Copy, Clone)]
    struct Vertex {
        position: [f32; 2],
    }
    implement_vertex!(Vertex, position);


pub fn test_glium_2d() {
    use crate::vector::Vec2;
    let (mut events_loop, display) = init_glium();


    let shape = vec![Vec2::new(-0.5, -0.5),Vec2::new(-0.0, 0.5),Vec2::new( 0.5, -0.25)];
    let shape : Vec<Vertex> = shape.iter().map(|v| Vertex{position : *v.get_arr() as [f32 ; 2]}).collect();

    let vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

    let vertex_shader_src = r#"
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
    let fragment_shader_src = r#"
    #version 140

    in vec2 my_attr;
    out vec4 color;

    void main() {
        color = vec4(my_attr,0.0,1.0);
    }
    "#;

    let program = glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None).unwrap();

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
            [0., 0., 0., 1.032]];
        let uniforms = uniform! {
            matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [ t , 0.0, 0.0, 1.0f32],
            ],
            perspective : perspective
        };

        target.clear_color(0.0,0.0,1.0,1.0);
        target.draw(&vertex_buffer, &indices, &program, &uniforms,
            &Default::default()).unwrap();
        target.finish().unwrap();
        
        listen_events(&mut events_loop, &mut closed)
    }
}