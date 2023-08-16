pub mod custom_events;
pub mod input_to_transform;
pub mod key_map; // this can be private when we're not debugging
pub mod saveload_dialog;
mod selection;
pub mod systems;
mod update_camera;

pub use selection::*;
pub use update_camera::*;
use winit::event::{ElementState, KeyboardInput};
use winit::event_loop::EventLoopProxy;

use std::collections::HashSet;

use winit::dpi::PhysicalPosition;

use winit::event::VirtualKeyCode as VKC;
use winit::event::{MouseScrollDelta, TouchPhase};

use winit_input_helper::WinitInputHelper;

use crate::vector::{Field, VecIndex};

use winit::event::{Event, WindowEvent};

use self::custom_events::CustomEvent;
use self::key_map::{
    KeyCombo, LOAD_LEVEL, MOVEMENT_MODE, QUIT, SAVE_LEVEL, TOGGLEABLE_KEYS, TOGGLE_DIMENSION,
};
use self::saveload_dialog::{request_load, request_save};

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

impl Input {
    pub fn listen_inputs(&mut self, event_loop_proxy: &EventLoopProxy<CustomEvent>) {
        self.toggle_keys.update_toggle_keys(&self.helper);
        if self.helper.key_released(QUIT) {
            event_loop_proxy
                .send_event(CustomEvent::Quit)
                .unwrap_or_default()
        }
        if self.helper.key_released(TOGGLE_DIMENSION) {
            event_loop_proxy
                .send_event(CustomEvent::SwapEngine)
                .unwrap_or_default()
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
    pub fn listen_events(
        &mut self,
        dim: VecIndex,
        event_loop_proxy: &EventLoopProxy<CustomEvent>,
        ev: &Event<CustomEvent>,
    ) {
        let update = &mut self.update;

        if let Event::WindowEvent { event, .. } = ev {
            match event {
                WindowEvent::CloseRequested => event_loop_proxy
                    .send_event(CustomEvent::Quit)
                    .unwrap_or_default(),
                WindowEvent::Resized(_) => *update = true,
                WindowEvent::Touch(winit::event::Touch { phase, .. }) => match phase {
                    winit::event::TouchPhase::Started => (),
                    winit::event::TouchPhase::Ended => (),
                    _ => (),
                },
                e => mouse_event(&mut self.mouse, e),
            }
        }
        if self.save_requested(ev) {
            self.last_movement_mode = self.movement_mode;
            self.movement_mode = MovementMode::Dialog;
            request_save(dim, event_loop_proxy);
        }
        if self.load_requested(ev) {
            self.last_movement_mode = self.movement_mode;
            self.movement_mode = MovementMode::Dialog;
            request_load(event_loop_proxy);
        }
        if self.helper.update(ev) {
            self.listen_inputs(event_loop_proxy);
        }
    }
    pub fn is_mouse_locked(&self) -> bool {
        matches!(
            self.movement_mode,
            MovementMode::Player(PlayerMovementMode::Mouse) | MovementMode::Shape(_)
        )
    }
    fn save_requested(&self, event: &Event<CustomEvent>) -> bool {
        self.key_combo(SAVE_LEVEL, event)
    }
    fn load_requested(&self, event: &Event<CustomEvent>) -> bool {
        self.key_combo(LOAD_LEVEL, event)
    }
    fn key_combo(&self, key_combo: KeyCombo, event: &Event<CustomEvent>) -> bool {
        // we explicitly event match here to get one event; winit helper generates multiple events
        // TODO: make this look nicer?
        if matches!(event,
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput {
                    input: KeyboardInput {
                        state: ElementState::Released,
                        virtual_keycode: Some(ref key),
                        ..
                    },
                    ..
                },
                ..
            } if (*key as usize) == (key_combo.release as usize))
        {
            self.helper.key_held(key_combo.hold)
        } else {
            false
        }
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
        WindowEvent::MouseWheel { delta, phase, .. } => match phase {
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
        },
        _ => (),
    }
}
