use glium::glutin::event::VirtualKeyCode as VKC;

use crate::vector::VecIndex;

pub const MOVEMENT_MODE: VKC = VKC::M;
pub const CANCEL_MANIPULATION: VKC = VKC::Backslash;
pub const TRANSLATE_MODE: VKC = VKC::T;
pub const ROTATE_MODE: VKC = VKC::R;
pub const SCALE_MODE: VKC = VKC::Y;
pub const FREE_MODE: VKC = VKC::F;

pub const AXIS_KEYMAP: [(VKC, VecIndex); 4] = [(VKC::Key1, 0), (VKC::Key2, 1), (VKC::Key3, 2), (VKC::Key4, 3)];

pub const CREATE_SHAPE: VKC = VKC::Period;

pub const TOGGLE_CLIPPING: VKC = VKC::C;



pub const QUIT: VKC = VKC::Q;
pub const TOGGLE_DIMENSION: VKC = VKC::Back;

//(- key, + key, axis)
pub const MOVE_KEYMAP : [(VKC,VKC, VecIndex); 3] = [
    (VKC::A, VKC::D, 0),
    (VKC::K, VKC::I, 1),
    (VKC::Q, VKC::E, 2),
];

pub const MOVE_FORWARDS: VKC = VKC::W;
pub const MOVE_BACKWARDS: VKC = VKC::S;

pub const PRINT_DEBUG: VKC = VKC::Space;