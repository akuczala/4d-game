use glium::glutin;
use glutin::event::VirtualKeyCode as VKC;
use glutin::event::ElementState::{Pressed,Released};

use std::time::Duration;

use crate::camera::Camera;
use crate::vector::{VectorTrait,MatrixTrait,Field};
use crate::geometry::Shape;

use crate::fps::FPSFloat;

use crate::game::Game;

use glutin::event::{Event,WindowEvent};

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
    //pub events_loop : EventLoop<()>,
    pub closed : bool,
    pub swap_engine : bool,
    pub update : bool,
}
impl Input {

    pub fn new() -> Self {
        Input{
            pressed : ButtonsPressed::new(),
            //events_loop : events_loop,
            closed : false,
            swap_engine : false,
            update : true
        }
    }
    //const SPEED : Field = 0.01;

    pub fn update_camera<V : VectorTrait>(&mut self, camera : &mut Camera<V>,
        frame_duration : FPSFloat)
    {
        //let frame_time = duration_as_field(frame_duration) as Field;
        let frame_time = frame_duration as Field;
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
                camera.spin(0,-1,frame_time);
                self.update = true;
            }
            if self.pressed.a {
                camera.spin(0,-1,-frame_time);
                self.update = true;
            }
            if self.pressed.i {
                camera.spin(1,-1,frame_time);
                self.update = true;
            }
            if self.pressed.k {
                camera.spin(1,-1,-frame_time);
                self.update = true;
            }
            if self.pressed.shift {
                if self.pressed.j {
                camera.spin(0,2,-frame_time);
                self.update = true;
                }
                if self.pressed.l {
                    camera.spin(0,2,frame_time);
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
                camera.spin(2,-1,-frame_time);
                self.update = true;
                }
                if self.pressed.l {
                    camera.spin(2,-1,frame_time);
                    self.update = true;
                }
                
            }
            
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

    pub fn print_debug<V : VectorTrait>(&mut self, game : &mut Game<V>, frame_seconds : FPSFloat)
    {
        if !self.pressed.space && !self.pressed.shift {
            //println!("camera.pos = {}",camera.pos);
            //rintln!("camera.heading = {}",camera.heading);
            //println!("camera.frame = {}",camera.frame);
            //println!("game time elapsed: {}", duration_as_field(game_time));
            //let frame_seconds = duration_as_field(frame_len);
            println!("frame time: {}, fps: {}", frame_seconds,1.0/frame_seconds);
            //clipping::print_in_front(&clip_state.in_front);
            //clip_state.print_debug();
            //clipping::test_dyn_separate(&shapes,&camera.pos);
            self.pressed.space = true;

        }
        //toggle clipping
        if !self.pressed.c {
            game.clip_state.clipping_enabled = !game.clip_state.clipping_enabled;
            println!("clipping={}",game.clip_state.clipping_enabled);
            self.pressed.c = true;
            self.update = true;
        }
    }
}
macro_rules! match_press {
    ( $( $x:expr ),* ) => {
        {
            $(
                Some($x) => pressed.$x = pressed_state,
            )*
        }
    };
}

impl Input {
    // listing the events produced by application and waiting to be received
    pub fn listen_events(&mut self, ev : &Event<()>) {
        let closed = &mut self.closed;
        let pressed = &mut self.pressed;
        let update = &mut self.update;
        let swap_engine = &mut self.swap_engine;

        match ev {
            Event::WindowEvent { event, .. } => match event {

                WindowEvent::CloseRequested => *closed = true,
                WindowEvent::Resized(_) => *update = true,
                WindowEvent::KeyboardInput{input, ..} => match input {
                	glutin::event::KeyboardInput{ virtual_keycode, state, ..} => {
                        let pressed_state = match state {
                            Pressed => true,
                            Released => false,
                        };
                        match virtual_keycode {
                    		Some(VKC::Escape) => *closed = !pressed_state,
                            Some(VKC::Back) => {
                                *swap_engine = !pressed_state;
                                
                                },
                            //match_press![W,S,A,D]
                            // use VKC{Space,W,S,A,D,I,K,J,L,T,C,LAlt,LShift};
                            // for k in vec![Space,W,S,A,D,I,K,J,L,T,C,LAlt,LShift]
                            // {
                            //     Some(k) => 
                            // }
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
                WindowEvent::Touch(glutin::event::Touch{phase, ..}) => match phase {
                        glutin::event::TouchPhase::Started => pressed.being_touched = true,
                        glutin::event::TouchPhase::Ended => pressed.being_touched = false,
                        _ => (),

                    }
                _ => (),
            },
            _ => (),
        }
    }

}
