use glium::Surface;
use super::init_glium;
use super::listen_events;

use crate::vector::{VectorTrait,Vec3};

#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 3],
}
implement_vertex!(Vertex, position);

pub const VERTEX_SHADER_SRC: &str = r#"
        #version 140
        in vec3 position;
        in vec3 normal;

        out vec3 v_normal;

        uniform mat4 perspective;
        uniform mat4 view;
        uniform mat4 model;
        void main() {
            mat4 modelview = view * model;
            v_normal = transpose(inverse(mat3(modelview))) * normal;
            gl_Position = perspective * modelview * vec4(position, 1.0);
        }
    "#;

pub const FRAGMENT_SHADER_SRC : &str = r#"
    #version 140
    out vec4 color;
    void main() {
        color = vec4(0.0, 1.0, 0.0, 1.0);
    }
"#;

fn build_perspective_mat<S>(target : &S) -> [[f32 ; 4] ; 4]
where S : Surface
{
    let (width, height) = target.get_dimensions();
    let aspect_ratio = height as f32 / width as f32;

    let fov: f32 = 3.141592 / 3.0;
    let zfar = 1024.0;
    let znear = 0.1;

    let f = 1.0 / (fov / 2.0).tan();

    [
        [f *   aspect_ratio   ,    0.0,              0.0              ,   0.0],
        [         0.0         ,     f ,              0.0              ,   0.0],
        [         0.0         ,    0.0,  (zfar+znear)/(zfar-znear)    ,   1.0],
        [         0.0         ,    0.0, -(2.0*zfar*znear)/(zfar-znear),   0.0],
    ]
}
fn build_view_matrix(position: &[f32; 3], direction: &[f32; 3], up: &[f32; 3]) -> [[f32; 4]; 4] {
    let f = {
        let f = direction;
        let len = f[0] * f[0] + f[1] * f[1] + f[2] * f[2];
        let len = len.sqrt();
        [f[0] / len, f[1] / len, f[2] / len]
    };

    let s = [up[1] * f[2] - up[2] * f[1],
             up[2] * f[0] - up[0] * f[2],
             up[0] * f[1] - up[1] * f[0]];

    let s_norm = {
        let len = s[0] * s[0] + s[1] * s[1] + s[2] * s[2];
        let len = len.sqrt();
        [s[0] / len, s[1] / len, s[2] / len]
    };

    let u = [f[1] * s_norm[2] - f[2] * s_norm[1],
             f[2] * s_norm[0] - f[0] * s_norm[2],
             f[0] * s_norm[1] - f[1] * s_norm[0]];

    let p = [-position[0] * s_norm[0] - position[1] * s_norm[1] - position[2] * s_norm[2],
             -position[0] * u[0] - position[1] * u[1] - position[2] * u[2],
             -position[0] * f[0] - position[1] * f[1] - position[2] * f[2]];

    [
        [s_norm[0], u[0], f[0], 0.0],
        [s_norm[1], u[1], f[1], 0.0],
        [s_norm[2], u[2], f[2], 0.0],
        [p[0], p[1], p[2], 1.0],
    ]
}

pub fn test_glium_3d() {
    use crate::geometry::buildshapes::build_cylinder;
    use super::ButtonsPressed;
    let mut pressed = ButtonsPressed::new();

    let (mut events_loop, display) = init_glium();

    //cylinder
    let cylinder = build_cylinder(50.0,100.0,32);
    let vertices : Vec<Vertex> = cylinder.verts.iter().map(|v| Vertex{position : *v.get_arr() as [f32 ; 3]}).collect();

    let positions = glium::VertexBuffer::new(&display, &vertices).unwrap();

    //let test_edgeis = vec![0,1,1,2,3,3,4,0u16];
    let mut test_vertis : Vec<u16> = Vec::new(); 
    for edge in cylinder.edges.iter() {
        test_vertis.push(edge.0 as u16);
        test_vertis.push(edge.1 as u16);
    }

    let indices = glium::IndexBuffer::new(
        &display,
        glium::index::PrimitiveType::LinesList,&test_vertis).unwrap();

    

    let program = glium::Program::from_source(&display, VERTEX_SHADER_SRC, FRAGMENT_SHADER_SRC,
                                              None).unwrap();
    let mut pos = Vec3::new(0.0,0.0,0.0);
    let mut closed = false;
    let dz = 0.001;
    while !closed {
        let mut target = display.draw();
        target.clear_color(0.05, 0.0, 0.05, 1.0);

        let model = [
            [0.01, 0.0, 0.0, 0.0],
            [0.0, 0.01, 0.0, 0.0],
            [0.0, 0.0, 0.01, 0.0],
            [pos[0], pos[1], pos[2], 1.0f32]
        ];
        let perspective = build_perspective_mat(&target);
        
        let view = build_view_matrix(&[1.0, 1.0, -2.0], &[-1.0,-1.0,2.0], &[0.0, 1.0, 0.0]);

        target.draw(&positions, &indices, &program,
            &uniform! { model: model, view : view, perspective : perspective },
                    &Default::default()).unwrap();
        target.finish().unwrap();

        listen_events(&mut events_loop,&mut closed, &mut pressed);
        if pressed.w {
            pos = pos + Vec3::new(0.0,0.0,dz);
        }
        if pressed.s {
            pos = pos - Vec3::new(0.0,0.0,dz);
        }
        if pressed.d {
            pos = pos + Vec3::new(dz,0.0,0.0);
        }
        if pressed.a {
            pos = pos - Vec3::new(dz,0.0,0.0);
        }

        
    } 

}