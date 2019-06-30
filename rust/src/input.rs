use glium::glutin;
use glutin::VirtualKeyCode as VKC;
use glutin::ElementState::{Pressed,Released};

use std::time::Duration;

use crate::draw::Camera;
use crate::vector::{VectorTrait,MatrixTrait,Field};
use crate::geometry::Shape;
use crate::clipping;
pub struct ButtonsPressed {
    pub w : bool,
    pub s : bool,
    pub a : bool,
    pub d : bool,
    pub i : bool,
    pub k : bool,
    pub t : bool,
    pub j : bool,
    pub l : bool,
    pub c : bool,
    pub space : bool,
    pub alt : bool,
    pub shift : bool,
    pub being_touched : bool
}
impl ButtonsPressed {
    pub fn new() -> Self{
        ButtonsPressed {
            w: false, s: false,
            a : false, d : false,
            i : false, k : false,
            j : false, l : false,
            t : true, //toggle transparency on key up
            c : true, //toggle clipping on key up
            space : false, alt : false, shift : false,
            being_touched : false,
        }
    }
}

fn duration_as_field(duration : &Duration) -> f32 {
 (duration.as_secs() as Field) + 0.001*(duration.subsec_millis() as Field)
}

pub struct Input {
    pub pressed : ButtonsPressed,
    pub events_loop : winit::EventsLoop,
    pub closed : bool,
    pub swap_engine : bool,
    pub update : bool,
}
impl Input {

    pub fn new(events_loop : winit::EventsLoop) -> Self {
        Input{
            pressed : ButtonsPressed::new(),
            events_loop : events_loop,
            closed : false,
            swap_engine : false,
            update : true
        }
    }
    //const SPEED : Field = 0.01;

    pub fn update_camera<V : VectorTrait>(&mut self, camera : &mut Camera<V>,
        frame_duration : &Duration)
    {
        let frame_time = duration_as_field(frame_duration) as Field;
        //fowards + backwards
        if self.pressed.w {
            camera.slide(camera.heading,frame_time);
            self.update = true;
        }
        if self.pressed.s {
            camera.slide(-camera.heading,frame_time);
            self.update = true;
        }
        if self.pressed.alt {
            //translation
            if self.pressed.d {
            camera.slide(camera.frame[0],frame_time);
            self.update = true;
            }
            if self.pressed.a {
            camera.slide(-camera.frame[0],frame_time);
            self.update = true;
            }
            if self.pressed.i {
            camera.slide(camera.frame[1],frame_time);
            self.update = true;
            }
            if self.pressed.k {
            camera.slide(-camera.frame[1],frame_time);
            self.update = true;
            }
            if self.pressed.j {
            camera.slide(-camera.frame[2],frame_time);
            self.update = true;
            }
            if self.pressed.l {
            camera.slide(camera.frame[2],frame_time);
            self.update = true;
            }
        } else {
           //rotation
            if self.pressed.d {
                camera.rotate(0,-1,frame_time);
                self.update = true;
            }
            if self.pressed.a {
                camera.rotate(0,-1,-frame_time);
                self.update = true;
            }
            if self.pressed.i {
                camera.rotate(1,-1,frame_time);
                self.update = true;
            }
            if self.pressed.k {
                camera.rotate(1,-1,-frame_time);
                self.update = true;
            }
            if self.pressed.shift {
                if self.pressed.j {
                camera.rotate(0,2,-frame_time);
                self.update = true;
                }
                if self.pressed.l {
                    camera.rotate(0,2,frame_time);
                    self.update = true;
                }
                //reset orientation
                if !self.pressed.space {
                    camera.frame = V::M::id();
                    camera.update();
                    self.update = true;
                    self.pressed.space = true;
                }
            } else {
                if self.pressed.j {
                camera.rotate(2,-1,-frame_time);
                self.update = true;
                }
                if self.pressed.l {
                    camera.rotate(2,-1,frame_time);
                    self.update = true;
                }
                
            }
            
        }

        //toggle clipping
        if !self.pressed.c {
            camera.clipping = !camera.clipping;
            println!("clipping={}",camera.clipping);
            self.pressed.c = true;
            self.update = true;
        }
    }
    pub fn update_shape<V : VectorTrait>(&mut self, shape : &mut Shape<V>)
    {
        //toggle transparency
        if !self.pressed.t {
            shape.transparent = !shape.transparent;
            self.pressed.t = true;
            self.update = true;
        }
    }

    pub fn print_debug<V : VectorTrait>(&mut self, camera : &Camera<V>,
        game_time : &Duration, frame_time : &Duration,
        in_front : &Vec<Vec<bool>>, shapes : &Vec<Shape<V>>)
    {
        if !self.pressed.space && !self.pressed.shift {
            println!("camera.pos = {}",camera.pos);
            println!("camera.heading = {}",camera.heading);
            println!("camera.frame = {}",camera.frame);
            println!("game time elapsed: {}", duration_as_field(game_time));
            let frame_seconds = duration_as_field(frame_time);
            println!("frame time: {}, fps: {}", frame_seconds,1.0/frame_seconds);
            //clipping::print_in_front(in_front);
            //clipping::test_dyn_separate(&shapes,&camera.pos);
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
        let swap_engine = &mut self.swap_engine;
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
                            		Some(VKC::Escape) => *closed = !pressed_state,
                                    Some(VKC::Back) => {
                                        *swap_engine = !pressed_state;
                                        
                                        },
        							Some(VKC::Space) => pressed.space = pressed_state,
                            		Some(VKC::W) => pressed.w = pressed_state,
                            		Some(VKC::S) => pressed.s = pressed_state,
                            		Some(VKC::A) => pressed.a = pressed_state,
                            		Some(VKC::D) => pressed.d = pressed_state,
                                    Some(VKC::I) => pressed.i = pressed_state,
                                    Some(VKC::K) => pressed.k = pressed_state,
                                    Some(VKC::J) => pressed.j= pressed_state,
                                    Some(VKC::L) => pressed.l = pressed_state,
                                    Some(VKC::T) => pressed.t = pressed_state,
                                    Some(VKC::C) => pressed.c = pressed_state,
                                    Some(VKC::LAlt) => pressed.alt = pressed_state,
                                    Some(VKC::LShift) => pressed.shift = pressed_state,
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
