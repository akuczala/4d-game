pub mod custom_events;
pub mod input_to_transform;
pub mod key_map; // this can be private when we're not debugging
pub mod saveload_dialog;
mod selection;
pub mod systems;
mod update_camera;

use glium::glutin::event::{ElementState, KeyboardInput};
pub use selection::*;
pub use update_camera::*;

use std::collections::HashSet;

use glium::glutin;
use glium::glutin::dpi::PhysicalPosition;

use glutin::event::VirtualKeyCode as VKC;
use glutin::event::{MouseScrollDelta, TouchPhase};

use winit_input_helper::WinitInputHelper;

use crate::components::*;
use crate::vector::{Field, VectorTrait};

use glutin::event::{Event, WindowEvent};

use self::custom_events::CustomEvent;
use self::key_map::{
    MOVEMENT_MODE, PRINT_DEBUG, QUIT, TOGGLEABLE_KEYS, TOGGLE_CLIPPING, TOGGLE_DIMENSION,
};

// fn duration_as_field(duration : &Duration) -> f32 {
//  (duration.as_secs() as Field) + 0.001*(duration.subsec_millis() as Field)
// }

//maximum allowed time step (player movement slows down at around 20 FPS)
//this prevents the player from jumping through walls
//we could be a little more careful with the nearest face detection
//right now, if the player ends up inside a wall, the nearest face is misidentified, because it's a signed distance
//see shape.rs
const MAX_DT: Field = 1000. / 20.;

const MOUSE_SENSITIVITY: Field = 0.2;
const MOUSE_STICK_POINT: [f32; 2] = [100., 100.];
const MOUSE_INTEGRATION_MAX: f32 = 1e6;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum PlayerMovementMode {
    Tank,
    Mouse,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ShapeMovementMode {
    Translate,
    Rotate,
    Scale,
    Free,
}

#[derive(Copy, Clone)]
pub enum MovementMode {
    Player(PlayerMovementMode),
    Shape(ShapeMovementMode),
    Dialog,
} //add flying tank mode, maybe flying mouse mode

#[derive(Default)]
pub struct MouseData {
    pub mouse_dpos: (f32, f32),
    pub scroll_dpos: Option<(f32, f32)>,
    pub integrated_mouse_dpos: (f32, f32),
    pub integrated_scroll_dpos: (f32, f32),
}
impl MouseData {
    pub fn mouse_or_scroll_deltas(&self) -> (f32, f32) {
        let (mut dx, mut dy) = self.mouse_dpos;
        if let Some((sdx, sdy)) = self.scroll_dpos {
            dx += sdx;
            dy += sdy;
        }
        (dx, dy)
    }
}

#[derive(Default)]
pub struct ToggleKeys(HashSet<VKC>);
impl ToggleKeys {
    fn update_toggle_keys(&mut self, helper: &WinitInputHelper) -> bool {
        let mut update = false;
        for key in TOGGLEABLE_KEYS {
            if helper.key_released(key) {
                update = true;
                if self.0.contains(&key) {
                    self.0.remove(&key);
                } else {
                    self.0.insert(key);
                }
            }
        }
        update
    }
    pub fn contains(&self, key: VKC) -> bool {
        self.0.contains(&key)
    }
    pub fn remove(&mut self, key: VKC) -> bool {
        self.0.remove(&key)
    }
    /// If contains key, remove key, run f, and return Some(result)
    /// Otherwise return None
    pub fn trigger_once<A, F>(&mut self, key: VKC, f: F) -> Option<A>
    where
        F: FnOnce() -> A,
    {
        self.trigger_once_bind(key, || Some(f()))
    }
    /// If contains key, remove key, run f and return result
    /// Otherwise return None
    pub fn trigger_once_bind<A, F>(&mut self, key: VKC, f: F) -> Option<A>
    where
        F: FnOnce() -> Option<A>,
    {
        if self.contains(key) {
            self.remove(key);
            f()
        } else {
            None
        }
    }
}

// TODO: better organized input modes + states
pub struct Input {
    pub helper: WinitInputHelper,
    //pub pressed : ButtonsPressed,
    pub closed: bool,
    pub swap_engine: bool,
    pub update: bool,
    pub frame_duration: crate::fps::FPSFloat,
    pub mouse: MouseData,
    pub toggle_keys: ToggleKeys,
    pub movement_mode: MovementMode,
    pub last_movement_mode: MovementMode,
}
impl Default for Input {
    fn default() -> Self {
        Self::new()
    }
}
impl Input {
    pub fn new() -> Self {
        Input {
            helper: WinitInputHelper::new(),
            //pressed : ButtonsPressed::new(),
            closed: false,
            swap_engine: false,
            update: true,
            frame_duration: crate::fps::TARGET_FPS,
            mouse: Default::default(),
            toggle_keys: Default::default(),
            movement_mode: MovementMode::Player(PlayerMovementMode::Mouse),
            last_movement_mode: MovementMode::Player(PlayerMovementMode::Mouse),
        }
    }
    pub fn get_dt(&self) -> Field {
        (self.frame_duration as Field).min(MAX_DT)
    }
    pub fn is_camera_movement_enabled(&self) -> bool {
        !matches!(
            self.movement_mode,
            MovementMode::Shape(ShapeMovementMode::Free)
        )
    }
}

pub fn print_debug<V: VectorTrait>(input: &mut Input, clip_state: &mut ClipState<V>) {
    if input.helper.key_released(PRINT_DEBUG) {
        //println!("camera.pos = {}",camera.pos);
        //rintln!("camera.heading = {}",camera.heading);
        //println!("camera.frame = {}",camera.frame);
        //println!("game time elapsed: {}", duration_as_field(game_time));
        //let frame_seconds = duration_as_field(frame_len);
        println!(
            "frame time: {}, fps: {}",
            input.frame_duration,
            1.0 / input.frame_duration
        );
        //clipping::print_in_front(&clip_state.in_front);
        //clip_state.print_debug();
        //clipping::test_dyn_separate(&shapes,&camera.pos);
        //input.pressed.space = true;
    }
    //toggle clipping
    if input.helper.key_released(TOGGLE_CLIPPING) {
        //TEMPORARILY DISABLED
        clip_state.clipping_enabled = !clip_state.clipping_enabled;
        println!("clipping={}", clip_state.clipping_enabled);
        //input.pressed.c = true;
        input.update = true;
    }
}

impl Input {
    pub fn listen_inputs(&mut self) {
        self.toggle_keys.update_toggle_keys(&self.helper);
        if self.helper.key_released(QUIT) {
            self.closed = true
        }
        if self.helper.key_released(TOGGLE_DIMENSION) {
            self.swap_engine = true
        }
        if self.helper.key_released(MOVEMENT_MODE) {
            self.movement_mode = match self.movement_mode {
                MovementMode::Player(player_mode) => MovementMode::Player(match player_mode {
                    PlayerMovementMode::Mouse => PlayerMovementMode::Tank,
                    PlayerMovementMode::Tank => PlayerMovementMode::Mouse,
                }),
                _ => MovementMode::Player(PlayerMovementMode::Mouse),
            }
        }
    }
    // run for every winit event
    pub fn listen_events<E>(&mut self, ev: &Event<E>) {
        let closed = &mut self.closed;
        let update = &mut self.update;

        if let Event::WindowEvent { event, .. } = ev {
            match event {
                WindowEvent::CloseRequested => *closed = true,
                WindowEvent::Resized(_) => *update = true,
                WindowEvent::Touch(glutin::event::Touch { phase, .. }) => match phase {
                    glutin::event::TouchPhase::Started => (),
                    glutin::event::TouchPhase::Ended => (),
                    _ => (),
                },
                e => mouse_event(&mut self.mouse, e),
            }
        }
        if self.helper.update(ev) {
            self.listen_inputs();
        }
    }
    pub fn is_mouse_locked(&self) -> bool {
        matches!(
            self.movement_mode,
            MovementMode::Player(PlayerMovementMode::Mouse) | MovementMode::Shape(_)
        )
    }
    pub fn save_requested(&self, event: &Event<CustomEvent>) -> bool {
        // we explicitly event match here to get one event; winit helper generates multiple events
        matches!(
            event,
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput {
                    input: KeyboardInput {
                        state: ElementState::Released,
                        virtual_keycode: Some(VKC::S),
                        ..
                    },
                    ..
                },
                ..
            }
        ) && self.helper.key_held(VKC::LWin)
    }
}
fn add_mod(x: &mut f32, dx: f32, x_max: f32) {
    *x = (*x + dx) % x_max
}

fn mouse_event(mouse: &mut MouseData, window_event: &WindowEvent) {
    match window_event {
        WindowEvent::CursorMoved { position, .. } => {
            mouse.mouse_dpos.0 = position.x as f32 - MOUSE_STICK_POINT[0];
            mouse.mouse_dpos.1 = position.y as f32 - MOUSE_STICK_POINT[1];
            add_mod(
                &mut mouse.integrated_mouse_dpos.0,
                mouse.mouse_dpos.0,
                MOUSE_INTEGRATION_MAX,
            );
            add_mod(
                &mut mouse.integrated_mouse_dpos.1,
                mouse.mouse_dpos.1,
                MOUSE_INTEGRATION_MAX,
            );
        }
        WindowEvent::MouseWheel { delta, phase, .. } => {
            match phase {
                TouchPhase::Started => {}
                TouchPhase::Moved => {
                    mouse.scroll_dpos = match delta {
                        MouseScrollDelta::LineDelta(x, y) => Some((*x, *y)),
                        MouseScrollDelta::PixelDelta(PhysicalPosition { x, y }) => {
                            Some((*x as f32, *y as f32))
                        }
                    };
                    let (dx, dy) = mouse.scroll_dpos.unwrap();
                    add_mod(
                        &mut mouse.integrated_scroll_dpos.0,
                        dx,
                        MOUSE_INTEGRATION_MAX,
                    );
                    add_mod(
                        &mut mouse.integrated_scroll_dpos.1,
                        dy,
                        MOUSE_INTEGRATION_MAX,
                    );
                }
                TouchPhase::Cancelled | TouchPhase::Ended => {
                    mouse.scroll_dpos = None;
                }
            }
            //println!("{:?}, {:?}",delta, phase)
        }
        _ => (),
    }
}
