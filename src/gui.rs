use crate::components::*;
use crate::ecs_utils::Componentable;
use crate::fps::FPSFloat;
use crate::geometry::shape::RefShapes;
use crate::input::{Input, ShapeManipulationMode, ShapeManipulationState};
use crate::vector::{MatrixTrait, VectorTrait};
use glium::glutin::event::{Event, WindowEvent};
use glium::glutin::event_loop::ControlFlow;
use glium::Display;
use glium::Frame;
use imgui::{Context, FontConfig, FontGlyphRanges, FontSource, ProgressBar, Ui};
use imgui_glium_renderer::Renderer;
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use map_in_place::MapVecInPlace;
use specs::prelude::*;
use std::time::Instant;
//mod clipboard;

#[derive(Default)]
struct State {
    text: String,
    checked: bool,
}
pub struct System {
    pub imgui: Context,
    pub platform: WinitPlatform,
    pub renderer: Renderer,
    pub font_size: f32,
    pub ui_args: UIArgs,
    state: State,
}

pub fn init(title: &str, display: &Display) -> System {
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

    System {
        imgui,
        platform,
        renderer,
        font_size,
        ui_args: UIArgs::None,
        state: State::default(),
    }
}
pub enum UIArgs {
    None,
    Test {
        frame_duration: FPSFloat,
        elapsed_time: u64,
        mouse_diff: (f32, f32),
        mouse_pos: Option<(f32, f32)>,
    },
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
        let mut debug_text = "".to_string();

        let player = world.read_resource::<Player>();
        // the let statements for storage here are needed to avoid temporary borrowing
        let maybe_target_storage = world.read_storage::<MaybeTarget<V>>();
        let maybe_target = maybe_target_storage
            .get(player.0)
            .expect("player has no target");
        let maybe_selected_storage = world.read_storage::<MaybeSelected>();
        let maybe_selected = maybe_selected_storage
            .get(player.0)
            .expect("player has no selection component");

        let shapes = world.read_component::<Shape<V>>();
        let transforms = world.read_component::<Transform<V, V::M>>();
        let mut normals: Vec<V> = vec![];
        for shape in (&shapes).join() {
            for face in shape.faces.iter() {
                normals.push(face.normal())
            }
        }
        let input = world.read_resource::<Input>();

        let debug_strings: Vec<String> = vec![
            format!(
                "Integrated mouse: {:?}\n",
                input.mouse.integrated_mouse_dpos
            ),
            format!(
                "Integrated scroll: {:?}\n",
                input.mouse.integrated_scroll_dpos
            ),
            match maybe_target {
                MaybeTarget(Some(target)) => format!(
                    "target: {}, {}, {}\n",
                    target.entity.id(),
                    target.distance,
                    target.point
                ),
                MaybeTarget(None) => "No target\n".to_string(),
            },
            match maybe_selected {
                MaybeSelected(Some(selected)) => {
                    //let bbox_storage = world.read_storage::<BBox<V>>();
                    //let selected_bbox = bbox_storage.get(selected.entity).expect("selected entity has no bbox");
                    let selected_transform = transforms.get(selected.entity).expect("Nope");
                    //let (frame, scaling) = selected_transform.decompose_rotation_scaling();
                    let (frame, scaling) = (selected_transform.frame, selected_transform.scale);
                    //let bbox_info = format!("target ({}) bbox: {:?}\n",selected.entity.id(), *selected_bbox);
                    let frame_info = format!(
                        "target frame: {}\n, {}\n{:?}\n",
                        selected.entity.id(),
                        frame,
                        scaling
                    );

                    let manip_state = world.read_resource::<ShapeManipulationState<V, V::M>>();
                    let manip_info = match manip_state.mode {
                        ShapeManipulationMode::Translate(v) => format!("Translate: {}", v),
                        ShapeManipulationMode::Rotate(a) => format!("Rotate: {:.2}", a),
                        ShapeManipulationMode::Scale(s) => format!("Scale: {:?}", s),
                        ShapeManipulationMode::Free(_t) => "Free".to_string(),
                    };
                    let axes_info = manip_state
                        .locked_axes
                        .iter()
                        .fold("Axes:".to_string(), |s, &i| s + &i.to_string())
                        + "\n";
                    let snap_info = format!("Snap: {}\n", manip_state.snap);
                    format!("{}{}{}{}", snap_info, axes_info, frame_info, manip_info)
                }
                MaybeSelected(None) => "No selection\n".to_string(),
            },
            //crate::clipping::ShapeClipState::<V>::in_front_debug(world),
        ]
        .into_iter()
        //.chain(all_verts.into_iter().map(|v| format!{"::{}\n", v}))
        //.chain(normals.into_iter().map(|n| format!("{}\n", n)))
        .collect();

        //print draw lines
        //let draw_lines = world.read_resource::<DrawLineList<V::SubV>>();
        // for line in draw_lines.0.iter() {
        //     if let Some(ref l) = line {
        //         debug_strings.push(format!("{:}\n",l.line));
        //     }
        // }

        //concatenate all strings
        for string in debug_strings.into_iter() {
            debug_text = format!("{}{}", debug_text, string);
        }

        Self::Debug {
            frame_duration,
            debug_text,
        }
    }
}

fn hello_world(_: &mut bool, ui: &mut Ui, ui_args: &mut UIArgs) {
    use imgui::{Condition, Window};
    ui.window("Debug info")
        .position([20.0, 20.0], Condition::Appearing)
        .size([300.0, 110.0], Condition::FirstUseEver)
        .build(|| {
            if let UIArgs::Test {
                ref frame_duration,
                ref elapsed_time,
                ref mouse_diff,
                ref mouse_pos,
            } = ui_args
            {
                ui.text(format!("FPS: {}", 1. / frame_duration));
                ui.text(format!("elapsed_time (ms): {}", elapsed_time));
                ui.text(format!("dmouse: {:?}", mouse_diff));
                ui.text(format!("mouse_pos: {:?}", mouse_pos));
            };
            ui.separator();
            let mouse_pos = ui.io().mouse_pos;
            ui.text(format!(
                "Mouse Position: ({:.1},{:.1})",
                mouse_pos[0], mouse_pos[1]
            ));
        });
}
fn simple_ui(_: &mut bool, ui: &mut Ui, ui_args: &mut UIArgs) {
    use imgui::{Condition, Window};
    ui.window("Press M to toggle mouse control")
        .position([0., 0.], Condition::Appearing)
        .size([190.0, 110.0], Condition::FirstUseEver)
        .bg_alpha(0.75)
        .title_bar(false)
        .resizable(false)
        .scroll_bar(false)
        .menu_bar(false)
        .build(|| {
            match ui_args {
                UIArgs::Test {
                    ref frame_duration,
                    ref elapsed_time,
                    ref mouse_diff,
                    ref mouse_pos,
                } => {
                    ui.text(format!("FPS: {0:0}", 1. / frame_duration));
                    ui.text(format!("elapsed_time (ms): {}", elapsed_time));
                    ui.text(format!("dmouse: {:?}", mouse_diff));
                    ui.text(format!("mouse_pos: {:?}", mouse_pos));
                }
                UIArgs::Simple {
                    ref frame_duration,
                    ref coins_collected,
                    ref coins_left,
                } => {
                    let total_coins = coins_left + coins_collected;
                    let coin_text = format!("Coins: {}/{}", coins_collected, total_coins);
                    ui.text(format!("FPS: {:0.0}", 1. / frame_duration));
                    ui.text(coin_text);
                    ProgressBar::new((*coins_collected as f32) / (total_coins as f32))
                        //.size([200.0, 20.0])
                        .build(ui);
                    ui.text("Press M to toggle mouse");
                    ui.text("Backspace toggles 3D/4D");
                }
                _ => (),
            };
        });
}
fn debug_ui(_: &mut bool, ui: &mut Ui, ui_args: &mut UIArgs, state: &mut State) {
    use imgui::{Condition, Window};
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
        });
}

impl System {
    pub fn update<E>(
        &mut self,
        display: &Display,
        last_frame: &mut Instant,
        event: &Event<E>,
        control_flow: &mut ControlFlow,
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
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            event => {
                let gl_window = display.gl_window();
                platform.handle_event(imgui.io_mut(), gl_window.window(), event);
            }
        }
    }
    pub fn draw(&mut self, display: &Display, target: &mut Frame) {
        let imgui = &mut self.imgui;
        let platform = &mut self.platform;
        let renderer = &mut self.renderer;
        //let font_size = self.font_size;
        let ui = imgui.frame();
        let mut run = true;
        match self.ui_args {
            UIArgs::Debug { .. } => debug_ui(&mut run, ui, &mut self.ui_args, &mut self.state),
            UIArgs::Simple { .. } => simple_ui(&mut run, ui, &mut self.ui_args),
            UIArgs::Test { .. } => hello_world(&mut run, ui, &mut self.ui_args),
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
