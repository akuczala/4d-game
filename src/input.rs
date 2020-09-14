use std::marker::PhantomData;

use glium::glutin;
use glutin::event::VirtualKeyCode as VKC;
use glutin::event::ElementState::{Pressed,Released};

use specs::{ReadStorage,WriteStorage,ReadExpect,WriteExpect,Read,Write,System,Join};

use std::time::Duration;

use crate::camera::Camera;
use crate::vector::{VectorTrait,MatrixTrait,Field};
use crate::geometry::Shape;
use crate::clipping::ClipState;
use crate::fps::FPSFloat;

//use crate::game::Game;

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
    pub frame_duration : crate::fps::FPSFloat
}
impl Default for Input {
    fn default() -> Self {
        Self::new()
    }
}
impl Input {

    pub fn new() -> Self {
        Input{
            pressed : ButtonsPressed::new(),
            //events_loop : events_loop,
            closed : false,
            swap_engine : false,
            update : true,
            frame_duration : crate::fps::TARGET_FPS,
        }
    }
}

pub struct UpdateCameraSystem<V : VectorTrait>(pub PhantomData<V>);
impl <'a,V : VectorTrait> System<'a> for UpdateCameraSystem<V> {
    type SystemData = (Write<'a,Input>,WriteExpect<'a,Camera<V>>);
    fn run(&mut self, (mut input, mut camera) : Self::SystemData) {
        update_camera(&mut input, &mut camera);
    }
}

pub fn update_camera<V : VectorTrait>(input : &mut Input, camera : &mut Camera<V>)
{
    //let frame_time = duration_as_field(frame_duration) as Field;
    let frame_time = input.frame_duration as Field;
    //fowards + backwards
    if input.pressed.w {
        camera.slide(camera.heading,frame_time);
        input.update = true;
    }
    if input.pressed.s {
        camera.slide(-camera.heading,frame_time);
        input.update = true;
    }
    if input.pressed.alt {
        //translation
        if input.pressed.d {
        camera.slide(camera.frame[0],frame_time);
        input.update = true;
        }
        if input.pressed.a {
        camera.slide(-camera.frame[0],frame_time);
        input.update = true;
        }
        if input.pressed.i {
        camera.slide(camera.frame[1],frame_time);
        input.update = true;
        }
        if input.pressed.k {
        camera.slide(-camera.frame[1],frame_time);
        input.update = true;
        }
        if input.pressed.j {
        camera.slide(-camera.frame[2],frame_time);
        input.update = true;
        }
        if input.pressed.l {
        camera.slide(camera.frame[2],frame_time);
        input.update = true;
        }
    } else {
       //rotation
        if input.pressed.d {
            camera.spin(0,-1,frame_time);
            input.update = true;
        }
        if input.pressed.a {
            camera.spin(0,-1,-frame_time);
            input.update = true;
        }
        if input.pressed.i {
            camera.spin(1,-1,frame_time);
            input.update = true;
        }
        if input.pressed.k {
            camera.spin(1,-1,-frame_time);
            input.update = true;
        }
        if input.pressed.shift {
            if input.pressed.j {
            camera.spin(0,2,-frame_time);
            input.update = true;
            }
            if input.pressed.l {
                camera.spin(0,2,frame_time);
                input.update = true;
            }
            //reset orientation
            if !input.pressed.space {
                camera.frame = V::M::id();
                camera.update();
                input.update = true;
                input.pressed.space = true;
            }
        } else {
            if input.pressed.j {
            camera.spin(2,-1,-frame_time);
            input.update = true;
            }
            if input.pressed.l {
                camera.spin(2,-1,frame_time);
                input.update = true;
            }
            
        }
        
    }

}
pub fn update_shape<V : VectorTrait>(input : &mut Input, shape : &mut Shape<V>)
{
    //toggle transparency
    if !input.pressed.t {
        shape.transparent = !shape.transparent;
        input.pressed.t = true;
        input.update = true;
    }
}

pub struct PrintDebugSystem<V : VectorTrait>(pub PhantomData<V>);
impl <'a,V : VectorTrait> System<'a> for PrintDebugSystem<V> {

    type SystemData = (Write<'a,Input>,Write<'a,ClipState<V>>);

    fn run(&mut self, (mut input, mut clip_state) : Self::SystemData) {
        print_debug::<V>(&mut input,&mut clip_state);
    }
}

pub fn print_debug<V : VectorTrait>(input : &mut Input,clip_state : &mut ClipState<V>)
{
    if !input.pressed.space && !input.pressed.shift {
        //println!("camera.pos = {}",camera.pos);
        //rintln!("camera.heading = {}",camera.heading);
        //println!("camera.frame = {}",camera.frame);
        //println!("game time elapsed: {}", duration_as_field(game_time));
        //let frame_seconds = duration_as_field(frame_len);
        println!("frame time: {}, fps: {}", input.frame_duration,1.0/input.frame_duration);
        //clipping::print_in_front(&clip_state.in_front);
        //clip_state.print_debug();
        //clipping::test_dyn_separate(&shapes,&camera.pos);
        input.pressed.space = true;

    }
    //toggle clipping
    if !input.pressed.c {
        //TEMPORARILY DISABLED
        clip_state.clipping_enabled = !clip_state.clipping_enabled;
        println!("clipping={}",clip_state.clipping_enabled);
        input.pressed.c = true;
        input.update = true;
    }
}
// macro_rules! match_press {
//     ( $( $x:expr ),* ) => {
//         {
//             $(
//                 Some($x) => pressed.$x = pressed_state,
//             )*
//         }
//     };
// }

impl Input {
    // listing the events produced by application and waiting to be received
    pub fn listen_events<E>(&mut self, ev : &Event<E>) {
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
