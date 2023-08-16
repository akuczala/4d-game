mod debug;
pub mod editor;
mod simple;

use crate::components::*;
use crate::constants::CUBE_LABEL_STR;
use crate::debug::make_debug_string;
use crate::ecs_utils::Componentable;
use crate::fps::FPSFloat;

use crate::vector::VectorTrait;

use glium::Display;
use glium::Frame;
use imgui::{Context, FontConfig, FontGlyphRanges, FontSource};
use imgui_glium_renderer::Renderer;
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use specs::prelude::*;
use std::time::Instant;
use winit::event::Event;

use self::debug::debug_gui;
use self::editor::editor_gui;
use self::simple::simple_gui;
//mod clipboard;

type ListBoxIndex = i32;

#[derive(Default)]
pub struct GuiState {
    text: String,
    checked: bool,
    item: ListBoxIndex,
}
impl GuiState {
    pub fn new(init_args: &GuiInitArgs) -> Self {
        Self {
            text: Default::default(),
            checked: Default::default(),
            item: init_args.default_shape_index,
        }
    }
    pub fn get_selected_shape_name(&self, init_args: &GuiInitArgs) -> ShapeLabel {
        ShapeLabel(init_args.shape_names[self.item as usize].clone())
    }
}

#[derive(Default)]
pub struct GuiInitArgs {
    shape_names: Vec<String>,
    default_shape_index: ListBoxIndex,
}

impl GuiInitArgs {
    pub fn new(shape_labels: &[ShapeLabel]) -> Self {
        let mut shape_names: Vec<String> =
            shape_labels.iter().map(|label| label.to_string()).collect();
        shape_names.sort();
        let cube_index = shape_names.iter().position(|name| name == CUBE_LABEL_STR);
        Self {
            shape_names,
            default_shape_index: cube_index.unwrap_or_default() as ListBoxIndex,
        }
    }
}
pub struct GuiSystem {
    pub imgui: Context,
    pub platform: WinitPlatform,
    pub renderer: Renderer,
    pub font_size: f32,
    pub ui_args: GuiArgs,
}

pub fn init(title: &str, display: &Display) -> GuiSystem {
    #[allow(unused)]
    let title = match title.rfind('/') {
        Some(idx) => title.split_at(idx + 1).1,
        None => title,
    };

    let mut imgui = Context::create();
    imgui.set_ini_filename(None);

    // if let Some(backend) = clipboard::init() {
    //     imgui.set_clipboard_backend(Box::new(backend));
    // } else {
    //     eprintln!("Failed to initialize clipboard");
    // }

    let mut platform = WinitPlatform::init(&mut imgui);
    {
        let gl_window = display.gl_window();
        let window = gl_window.window();
        platform.attach_window(imgui.io_mut(), window, HiDpiMode::Rounded);
    }

    let hidpi_factor = platform.hidpi_factor();
    let font_size = (13.0 * hidpi_factor) as f32;
    imgui.fonts().add_font(&[
        FontSource::DefaultFontData {
            config: Some(FontConfig {
                size_pixels: font_size,
                ..FontConfig::default()
            }),
        },
        FontSource::TtfData {
            data: include_bytes!("../resources/mplus-1p-regular.ttf"),
            size_pixels: font_size,
            config: Some(FontConfig {
                rasterizer_multiply: 1.75,
                glyph_ranges: FontGlyphRanges::japanese(),
                ..FontConfig::default()
            }),
        },
    ]);

    imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;

    let renderer = Renderer::init(&mut imgui, display).expect("Failed to initialize renderer");

    GuiSystem {
        imgui,
        platform,
        renderer,
        font_size,
        ui_args: GuiArgs::None,
    }
}
pub enum GuiArgs {
    None,
    Simple {
        frame_duration: FPSFloat,
        coins_collected: u32,
        coins_left: u32,
    },
    Debug {
        frame_duration: FPSFloat,
        debug_text: String,
    },
    Editor {
        info_string: String,
    },
}
impl GuiArgs {
    pub fn new_debug<V>(world: &World, frame_duration: FPSFloat) -> Self
    where
        V: VectorTrait + Componentable,
        V::M: Componentable,
    {
        Self::Debug {
            frame_duration,
            debug_text: make_debug_string::<V>(world),
        }
    }
}

impl GuiSystem {
    pub fn update<E>(
        &mut self,
        display: &Display,
        last_frame: &mut Instant,
        event: &Event<E>,
        ui_args: GuiArgs,
    ) {
        let imgui = &mut self.imgui;
        let platform = &mut self.platform;
        self.ui_args = ui_args;
        match event {
            Event::NewEvents(_) => {
                let now = Instant::now();
                imgui.io_mut().update_delta_time(now - *last_frame);
                *last_frame = now;
            }
            Event::MainEventsCleared => {
                let gl_window = display.gl_window();
                platform
                    .prepare_frame(imgui.io_mut(), gl_window.window())
                    .expect("Failed to prepare frame");
                gl_window.window().request_redraw();
            }
            event => {
                let gl_window = display.gl_window();
                platform.handle_event(imgui.io_mut(), gl_window.window(), event);
            }
        }
    }
    pub fn draw(
        &mut self,
        display: &Display,
        target: &mut Frame,
        state: &mut GuiState,
        init_args: &GuiInitArgs,
    ) {
        let imgui = &mut self.imgui;
        let platform = &mut self.platform;
        let renderer = &mut self.renderer;
        //let font_size = self.font_size;
        let ui = imgui.frame();
        let mut run = true;
        match self.ui_args {
            GuiArgs::Debug { .. } => debug_gui(&mut run, ui, &mut self.ui_args, state, init_args),
            GuiArgs::Simple { .. } => simple_gui(&mut run, ui, &mut self.ui_args),
            GuiArgs::Editor { ref info_string } => {
                editor_gui(&mut run, ui, state, init_args, info_string)
            }
            GuiArgs::None => (),
        };
        if !run {
            //*control_flow = ControlFlow::Exit;
            panic!("Would exit here because ui didn't run");
        }

        let gl_window = display.gl_window();

        //target.clear_color_srgb(1.0, 1.0, 1.0, 1.0);
        platform.prepare_render(ui, gl_window.window());
        let draw_data = imgui.render();
        renderer
            .render(target, draw_data)
            .expect("Rendering failed");
        //target.finish().expect("Failed to swap buffers");
    }
}
