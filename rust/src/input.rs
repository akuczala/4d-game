use glium::glutin;
use glutin::VirtualKeyCode as VKC;
use glutin::ElementState::{Pressed,Released};
use crate::draw::Camera;
use crate::vector::{VectorTrait,Field};
use crate::geometry::Shape;

pub struct ButtonsPressed {
    pub w : bool,
    pub s : bool,
    pub a : bool,
    pub d : bool,
    pub i : bool,
    pub k : bool,
    pub t : bool,
    pub space : bool,
    pub alt : bool,
    pub being_touched : bool
}
impl ButtonsPressed {
    pub fn new() -> Self{
        ButtonsPressed {
            w: false, s: false,
            a : false, d : false,
            i : false, k : false,
            t : true, //toggle transparency on key up
            space : false, alt : false,
            being_touched : false,
        }
    }
}

pub struct Input {
    pub pressed : ButtonsPressed,
    pub events_loop : winit::EventsLoop,
    pub closed : bool,
    pub update : bool,
}
impl Input {

    pub fn new(events_loop : winit::EventsLoop) -> Self {
        Input{
            pressed : ButtonsPressed::new(),
            events_loop : events_loop,
            closed : false,
            update : true
        }
    }
    //const SPEED : Field = 0.01;

    pub fn update_camera<Vec3>(&mut self, camera : &mut Camera<Vec3>)
    where Vec3 : VectorTrait
    {
        //fowards + backwards
        if self.pressed.w {
            camera.slide(camera.heading);
            self.update = true;
        }
        if self.pressed.s {
            camera.slide(-camera.heading);
            self.update = true;
        }
        if self.pressed.alt {
            //translation
            if self.pressed.d {
            camera.slide(camera.frame[0]);
            self.update = true;
            }
            if self.pressed.a {
            camera.slide(-camera.frame[0]);
            self.update = true;
            }
            if self.pressed.i {
            camera.slide(camera.frame[1]);
            self.update = true;
            }
            if self.pressed.k {
            camera.slide(-camera.frame[1]);
            self.update = true;
            }
        } else {
           //rotation
            if self.pressed.d {
                camera.rotate(0,-1,1.0);
                self.update = true;
            }
            if self.pressed.a {
                camera.rotate(0,-1,-1.0);
                self.update = true;
            }
            if self.pressed.i {
                camera.rotate(1,-1,1.0);
                self.update = true;
            }
            if self.pressed.k {
                camera.rotate(1,-1,-1.0);
                self.update = true;
            }
        }
    }
    pub fn update_shape<V>(&mut self, shape : &mut Shape<V>)
    where V : VectorTrait
    {
        //toggle transparency
        if !self.pressed.t {
            shape.transparent = !shape.transparent;
            self.pressed.t = true;
            self.update = true;
        }
    }
    pub fn print_debug<V>(&mut self, camera : &Camera<V>)
    where V : VectorTrait
    {
        if !self.pressed.space {
            println!("camera.pos = {}",camera.pos);
            println!("camera.heading = {}",camera.heading);
            println!("camera.frame = {}",camera.frame);
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
        let update = &mut self.update;
        events_loop.poll_events(|ev| {
                match ev {
                    glutin::Event::WindowEvent { event, .. } => match event {
                        glutin::WindowEvent::CloseRequested => *closed = true,
                        glutin::WindowEvent::Resized(_) => *update = true,
                        glutin::WindowEvent::KeyboardInput{input, ..} => match input {
                        	glutin::KeyboardInput{ virtual_keycode, state, ..} => {
                                let pressed_state = match state {
                                    Pressed => true,
                                    Released => false,
                                };
                                match virtual_keycode {
                            		Some(VKC::Escape) => *closed = true,
        							Some(VKC::Space) => pressed.space = pressed_state,
                            		Some(VKC::W) => pressed.w = pressed_state,
                            		Some(VKC::S) => pressed.s = pressed_state,
                            		Some(VKC::A) => pressed.a = pressed_state,
                            		Some(VKC::D) => pressed.d = pressed_state,
                                    Some(VKC::I) => pressed.i = pressed_state,
                                    Some(VKC::K) => pressed.k = pressed_state,
                                    Some(VKC::T) => pressed.t = pressed_state,
                                    Some(VKC::LAlt) => pressed.alt = pressed_state,
                            		_ => (),
                                }
                        	},
                        },
                        glutin::WindowEvent::Touch(glutin::Touch{phase, ..}) => match phase {
                                glutin::TouchPhase::Started => pressed.being_touched = true,
                                glutin::TouchPhase::Ended => pressed.being_touched = false,
                                _ => (),

                            }
                        _ => (),
                    },
                    _ => (),
                }
            });
    }

}
