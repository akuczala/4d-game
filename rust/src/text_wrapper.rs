extern crate glium;
extern crate glium_text;
extern crate cgmath;

use std::ops;

pub trait SizedFacade: glium::backend::Facade + Sized {}
impl<T> SizedFacade for T where T: glium::backend::Facade + Sized {}

pub struct System<'a, F: SizedFacade + 'a> {
    system: glium_text::TextSystem,
    display: & 'a F
}

impl<'a, F: SizedFacade + 'a> System<'a, F> {
    pub fn new(display: & 'a F) -> System<'a, F> {
        System{
            system: glium_text::TextSystem::new(display),
            display: display
        }
    }

    pub fn new_font(&self, bytes: &[u8], size:u32) -> Font<F> {

        Font {
            system: self,
            texture: glium_text::FontTexture::new(self.display, bytes, size).unwrap()
        }
    }
}

pub struct Font<'a, T: SizedFacade + 'a> {
    system: & 'a System<'a, T>,
    texture: glium_text::FontTexture,
}

impl<'a, T: SizedFacade + 'a> Font<'a, T> {
    pub fn new_text(&self, s: &str) -> Text<&glium_text::FontTexture, T> {
        Text {
            text: glium_text::TextDisplay::new(&self.system.system, &(self.texture), s),
            system: self.system,
        }
    }
}

pub struct Text<'a, T: ops::Deref<Target=glium_text::FontTexture>, F: SizedFacade + 'a> {
    text: glium_text::TextDisplay<T>,
    system: & 'a System<'a, F>,
}

impl<'a, T: ops::Deref<Target=glium_text::FontTexture>, F: SizedFacade + 'a> Text<'a, T, F> {
    pub fn draw(&self, frame: &mut glium::Frame, matrix: [[f32; 4]; 4], color: (f32, f32, f32, f32)){
        glium_text::draw(&self.text, &self.system.system, frame, matrix, color);
    }
}