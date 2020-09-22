use glium::Frame;
use glium::glutin;
use glium::glutin::event::{Event, WindowEvent};
use glium::glutin::event_loop::{ControlFlow};
use glium::glutin::window::WindowBuilder;
use glium::{Display, Surface};
use imgui::{Context, FontConfig, FontGlyphRanges, FontSource, Ui, ProgressBar};
use imgui_glium_renderer::Renderer;
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use std::time::Instant;
use crate::fps::FPSFloat;
//mod clipboard;

pub struct System {
    //pub event_loop: EventLoop<()>,
    //pub display: glium::Display,
    pub imgui: Context,
    pub platform: WinitPlatform,
    pub renderer: Renderer,
    pub font_size: f32,
    pub ui_args : UIArgs,
}

pub fn init(title: &str, display : &Display) -> System {
    let title = match title.rfind('/') {
        Some(idx) => title.split_at(idx + 1).1,
        None => title,
    };
    //let event_loop = EventLoop::new();
    // let context = glutin::ContextBuilder::new().with_vsync(true);
    // let builder = WindowBuilder::new()
    //     .with_title(title.to_owned())
    //     .with_inner_size(glutin::dpi::LogicalSize::new(1024f64, 768f64));
    // let display =
    //     Display::new(builder, context, &event_loop).expect("Failed to initialize display");

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
        platform.attach_window(imgui.io_mut(), &window, HiDpiMode::Rounded);
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

    System {
        //event_loop,
        //display,
        imgui,
        platform,
        renderer,
        font_size,
        ui_args : UIArgs::None,
    }
}
#[derive(Copy,Clone)]
pub enum UIArgs{
    None,
    Test{
        frame_duration : FPSFloat,
        elapsed_time : u64,
        mouse_diff : (f32,f32),
        mouse_pos : Option<(f32,f32)>,
    },
    Simple{
        frame_duration : FPSFloat,
        coins_collected : u32,
        coins_left : u32,
    }
}


fn hello_world(_ : &mut bool, ui : &mut Ui, ui_args : &mut UIArgs) {
        use imgui::{Window,im_str,Condition};
        Window::new(im_str!("Debug info"))
            .position([20.0, 20.0], Condition::Appearing)
            .size([300.0, 110.0], Condition::FirstUseEver)
            .build(ui, || {
                match ui_args {
                    UIArgs::Test{ref frame_duration, ref elapsed_time, ref mouse_diff, ref mouse_pos} => {
                        ui.text(format!("FPS: {}",1./frame_duration));
                        ui.text(format!("elapsed_time (ms): {}",elapsed_time));
                        ui.text(format!("dmouse: {:?}",mouse_diff));
                        ui.text(format!("mouse_pos: {:?}",mouse_pos));
                    }
                    _ => ()
                };
                ui.separator();
                let mouse_pos = ui.io().mouse_pos;
                ui.text(format!(
                    "Mouse Position: ({:.1},{:.1})",
                    mouse_pos[0], mouse_pos[1]
                ));
            });
        
    }
fn simple_ui(_ : &mut bool, ui : &mut Ui, ui_args : &mut UIArgs) {
        use imgui::{Window,im_str,Condition};
        Window::new(im_str!("Press M to toggle mouse control"))
            .position([0.,0.], Condition::Appearing)
            .size([190.0, 110.0], Condition::FirstUseEver)
            .bg_alpha(0.75)
            .title_bar(false)
            .resizable(false)
            .scroll_bar(false)
            .menu_bar(false)
            .build(ui, || {
                match ui_args {
                    UIArgs::None => (),
                    UIArgs::Test{ref frame_duration, ref elapsed_time, ref mouse_diff, ref mouse_pos} => {
                        ui.text(format!("FPS: {0:0}",1./frame_duration));
                        ui.text(format!("elapsed_time (ms): {}",elapsed_time));
                        ui.text(format!("dmouse: {:?}",mouse_diff));
                        ui.text(format!("mouse_pos: {:?}",mouse_pos));
                    }
                    UIArgs::Simple{ref frame_duration, ref coins_collected, ref coins_left} => {
                        let total_coins = coins_left + coins_collected;
                        let coin_text = format!("Coins: {}/{}",coins_collected,total_coins);
                        ui.text(format!("FPS: {:0.0}",1./frame_duration));
                        ui.text(coin_text);
                        ProgressBar::new(
                            (*coins_collected as f32)/(total_coins as f32))
                        //.size([200.0, 20.0])
                        .build(ui);
                        ui.text("Press M to toggle mouse");
                        ui.text("Backspace toggles 3D/4D");
                    }
                };
            });
        
    }

impl System {
    pub fn update<E>(&mut self, display : &Display, last_frame : &mut Instant, event : &Event<E>, control_flow : &mut ControlFlow, ui_args : UIArgs) {
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
                    .prepare_frame(imgui.io_mut(), &gl_window.window())
                    .expect("Failed to prepare frame");
                gl_window.window().request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            event => {
                let gl_window = display.gl_window();
                platform.handle_event(imgui.io_mut(), gl_window.window(), &event);
            }
        }
    }
    pub fn draw(&mut self, display : &Display, target : &mut Frame) {
        let imgui = &mut self.imgui;
        let platform = &mut self.platform;
        let renderer = &mut self.renderer;
        //let font_size = self.font_size;
        let mut ui = imgui.frame();
        let mut run = true;
        simple_ui(&mut run, &mut ui, &mut self.ui_args);
        if !run {
            //*control_flow = ControlFlow::Exit;
            panic!("Would exit here because ui didn't run");
        }

        let gl_window = display.gl_window();

        //target.clear_color_srgb(1.0, 1.0, 1.0, 1.0);
        platform.prepare_render(&ui, gl_window.window());
        let draw_data = ui.render();
        renderer
            .render(target, draw_data)
            .expect("Rendering failed");
        //target.finish().expect("Failed to swap buffers");
    }
}