use glium::glutin;
use glutin::VirtualKeyCode as VKC;
use glutin::ElementState::{Pressed,Released};
use crate::draw::Camera;
use crate::vector::{VectorTrait,Field};

pub struct ButtonsPressed {
    pub w : bool,
    pub s : bool,
    pub a : bool,
    pub d : bool,
    pub i : bool,
    pub k : bool,
    pub space : bool,
    pub alt : bool
}
impl ButtonsPressed {
    pub fn new() -> Self{
        ButtonsPressed {
            w: false, s: false,
            a : false, d : false,
            i : false, k : false,
            space : false, alt : false
        }
    }
}

pub struct Input {
    pub pressed : ButtonsPressed,
    pub events_loop : winit::EventsLoop,
    pub closed : bool,
}
impl Input {

    pub fn new(events_loop : winit::EventsLoop) -> Self {
        Input{
            pressed : ButtonsPressed::new(),
            events_loop : events_loop,
            closed : false
        }
    }
    //const SPEED : Field = 0.01;

    pub fn update_camera<Vec3>(&self, camera : &mut Camera<Vec3>) -> bool
    where Vec3 : VectorTrait
    {
        let mut update = false;
        //fowards + backwards
        if self.pressed.w {
            camera.slide(camera.heading);
            update = true;
        }
        if self.pressed.s {
            camera.slide(-camera.heading);
            update = true;
        }
        if self.pressed.alt {
            //translation
            if self.pressed.d {
            camera.slide(camera.frame[0]);
            update = true;
            }
            if self.pressed.a {
            camera.slide(-camera.frame[0]);
            update = true;
            }
            if self.pressed.i {
            camera.slide(camera.frame[1]);
            update = true;
            }
            if self.pressed.k {
            camera.slide(-camera.frame[1]);
            update = true;
            }
        } else {
           //rotation
            if self.pressed.d {
                camera.rotate(0,-1,-1.0);
                update = true;
            }
            if self.pressed.a {
                camera.rotate(0,-1,1.0);
                update = true;
            }
            if self.pressed.i {
                camera.rotate(1,-1,-1.0);
                update = true;
            }
            if self.pressed.k {
                camera.rotate(1,-1,1.0);
                update = true;
            }
        }
        update
    }
    pub fn print_debug<V>(&mut self, camera : &Camera<V>)
    where V : VectorTrait
    {
        if !self.pressed.space {
            println!("camera.pos = {}",camera.pos);
            self.pressed.space = true;
        }
    }
}
impl Input {
    // listing the events produced by application and waiting to be received
    pub fn listen_events(&mut self) {
        let events_loop = &mut self.events_loop;
        let closed = &mut self.closed;
        let pressed = &mut self.pressed;
        events_loop.poll_events(|ev| {
                match ev {
                    glutin::Event::WindowEvent { event, .. } => match event {
                        glutin::WindowEvent::CloseRequested => *closed = true,
                        glutin::WindowEvent::KeyboardInput{input, ..} => match input {
                        	glutin::KeyboardInput{ virtual_keycode, state, ..} => match (virtual_keycode, state) {
                        		(Some(VKC::Escape), Pressed) => *closed = true,
    							(Some(VKC::Space), Pressed) => pressed.space = true,
                        		(Some(VKC::Space), Released) => pressed.space = false,

                        		(Some(VKC::W), Pressed) => pressed.w = true,
                        		(Some(VKC::W), Released) => pressed.w = false,
                        		(Some(VKC::S), Pressed) => pressed.s = true,
                        		(Some(VKC::S), Released) => pressed.s = false,
                        		(Some(VKC::A), Pressed) => pressed.a = true,
                        		(Some(VKC::A), Released) => pressed.a = false,
                        		(Some(VKC::D), Pressed) => pressed.d = true,
                        		(Some(VKC::D), Released) => pressed.d = false,
                                (Some(VKC::I), Pressed) => pressed.i = true,
                                (Some(VKC::I), Released) => pressed.i = false,
                                (Some(VKC::K), Pressed) => pressed.k = true,
                                (Some(VKC::K), Released) => pressed.k = false,
                                (Some(VKC::LAlt), Pressed) => pressed.alt = true,
                                (Some(VKC::LAlt), Released) => pressed.alt = false,
                        		_ => (),

                        	},

                        },
                        _ => (),
                    },
                    _ => (),
                }
            });
    }

}
