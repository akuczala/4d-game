use crate::components::*;
use crate::constants::CUBE_LABEL_STR;
use crate::debug::make_debug_string;
use crate::ecs_utils::Componentable;
use crate::fps::FPSFloat;

use crate::vector::VectorTrait;
use glium::glutin::event::Event;

use glium::Display;
use glium::Frame;
use imgui::{Context, FontConfig, FontGlyphRanges, FontSource, ProgressBar, Ui};
use imgui_glium_renderer::Renderer;
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use specs::prelude::*;
use std::time::Instant;
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
    pub ui_args: UIArgs,
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
        ui_args: UIArgs::None,
    }
}
pub enum UIArgs {
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
}
impl UIArgs {
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

fn simple_ui(_: &mut bool, ui: &mut Ui, ui_args: &mut UIArgs) {
    use imgui::Condition;
    ui.window("Press M to toggle mouse control")
        .position([0., 0.], Condition::Appearing)
        .size([190.0, 110.0], Condition::FirstUseEver)
        .bg_alpha(0.75)
        .title_bar(false)
        .resizable(false)
        .scroll_bar(false)
        .menu_bar(false)
        .build(|| {
            if let UIArgs::Simple {
                ref frame_duration,
                ref coins_collected,
                ref coins_left,
            } = ui_args
            {
                let total_coins = coins_left + coins_collected;
                let coin_text = format!("Coins: {}/{}", coins_collected, total_coins);
                ui.text(format!("FPS: {:0.0}", 1. / frame_duration));
                ui.text(coin_text);
                ProgressBar::new((*coins_collected as f32) / (total_coins as f32))
                    //.size([200.0, 20.0])
                    .build(ui);
                ui.text("Press M to toggle mouse");
                ui.text("Backspace toggles 3D/4D");
            };
        });
}
fn debug_ui(
    _: &mut bool,
    ui: &mut Ui,
    ui_args: &mut UIArgs,
    state: &mut GuiState,
    init_args: &GuiInitArgs,
) {
    use imgui::Condition;
    ui.window("Press M to toggle mouse control")
        .position([0., 0.], Condition::Appearing)
        .size([190.0, 500.0], Condition::FirstUseEver)
        .always_auto_resize(true)
        .bg_alpha(0.75)
        .title_bar(false)
        .resizable(false)
        .scroll_bar(false)
        .menu_bar(false)
        .build(|| {
            if let UIArgs::Debug {
                ref frame_duration,
                ref debug_text,
            } = ui_args
            {
                ui.text(format!("FPS: {:0.0}", 1. / frame_duration));
                ui.text(debug_text);
            };
            if ui.radio_button_bool("I toggle my state on click", state.checked) {
                state.checked = !state.checked; // flip state on click
                state.text = "*** Toggling radio button was clicked".to_string();
            }
            let items = &init_args
                .shape_names
                .iter()
                .enumerate()
                // for some reason, this is what i need to do to get the imgui ids not to clash(?)
                // see https://github.com/ocornut/imgui/blob/master/docs/FAQ.md
                // but it shouldn't be this annoying
                // as an example, if we set the list to ["ok", "so"], we get the error???
                .map(|(i, s)| format!("{}##foo{}ffff", s, i))
                .collect::<Vec<_>>();
            let items = &items.iter().collect::<Vec<_>>();
            ui.list_box("Shape", &mut state.item, items, 10);
        });
}

impl GuiSystem {
    pub fn update<E>(
        &mut self,
        display: &Display,
        last_frame: &mut Instant,
        event: &Event<E>,
        ui_args: UIArgs,
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
            UIArgs::Debug { .. } => debug_ui(&mut run, ui, &mut self.ui_args, state, init_args),
            UIArgs::Simple { .. } => simple_ui(&mut run, ui, &mut self.ui_args),
            UIArgs::None => (),
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
