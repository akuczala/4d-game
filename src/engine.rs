mod dispatcher;
pub use dispatcher::get_engine_dispatcher_builder;

use glium::glutin::event_loop::EventLoopProxy;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::build_level::build_scene;
use crate::collide;
use crate::config::Config;
use crate::constants::FRAME_MS;
use crate::ecs_utils::Componentable;
use crate::graphics::{DefaultGraphics, GraphicsTrait};
use crate::gui::{GuiInitArgs, GuiState, GuiSystem};
use crate::input::{custom_events::CustomEvent, ShapeManipulationState};
use crate::saveload::save_level_to_file;
use crate::utils::ValidDimension;
use crate::FPSTimer;
use glium::Display;
use specs::prelude::*;
use std::marker::PhantomData;

use std::time::{Duration, Instant};

use crate::draw;
use crate::gui::UIArgs;
use glium::glutin::event::{Event, WindowEvent};
//NOTES:
// include visual indicator of what direction a collision is in

use crate::input::Input;

use crate::components::*;

use crate::vector::{Vec3, Vec4, VectorTrait};

// TODO: reduce number of explicit constraints needed by introducing a componentable-constrained trait?
pub struct EngineD<V, G> {
    pub world: World,
    graphics: G,
    gui: Option<crate::gui::GuiSystem>,
    dispatcher: Dispatcher<'static, 'static>,
    dummy: PhantomData<V>, //Forces EngineD to take V as a parameter
}
impl<V, G> EngineD<V, G>
where
    V: VectorTrait + Componentable + Serialize + DeserializeOwned,
    V::SubV: Componentable + Serialize + DeserializeOwned,
    V::M: Componentable + Serialize + DeserializeOwned,
    G: GraphicsTrait,
{
    pub fn new(config: &Config, graphics: G, maybe_gui: Option<GuiSystem>) -> Self {
        let mut world = World::new();

        let mut dispatcher = get_engine_dispatcher_builder::<V>().build();

        dispatcher.setup(&mut world);

        world.insert(Input::new());
        world.insert(ShapeManipulationState::default() as ShapeManipulationState<V, V::M>);
        world.insert(config.clone());

        build_scene::<V>(&mut world);
        Self::init_gui(&mut world);

        collide::create_spatial_hash::<V>(&mut world);

        world.insert(ClipState::<V>::new()); // decompose into single entity properties
        world.insert(draw::DrawLineList::<V>(vec![])); // unclear if this would be better as entities; might be able to thread
        world.insert(DrawLineList::<V::SubV>(vec![]));

        EngineD {
            world,
            dispatcher,
            graphics,
            gui: maybe_gui,
            dummy: PhantomData,
        }
    }

    fn init_gui(world: &mut World) {
        let init_args = GuiInitArgs::new(&world.fetch::<RefShapes<V>>().get_labels());
        world.insert(init_args);
        world.insert(GuiState::default());
    }

    //runs for each event
    pub fn update(
        &mut self,
        event: &Event<CustomEvent>,
        event_loop_proxy: &EventLoopProxy<CustomEvent>,
        display: &Display,
        fps_timer: &mut FPSTimer,
    ) {
        //input events
        self.world
            .write_resource::<Input>()
            .listen_events(V::DIM, event_loop_proxy, event);

        //window / game / redraw events
        match event {
            Event::UserEvent(custom_event) => match custom_event {
                CustomEvent::LoadDialog(maybe_file) => {
                    if let Some(file) = maybe_file {
                        println!("Loading level {}...", file.file_name());
                        event_loop_proxy
                            .send_event(CustomEvent::LoadLevel(file.path().to_owned()))
                            .unwrap_or_default();
                    }
                    let mut input = self.world.write_resource::<Input>();
                    input.movement_mode = input.last_movement_mode;
                }
                CustomEvent::SaveDialog(maybe_file) => {
                    if let Some(file) = maybe_file {
                        println!("Saving level to {}", file.file_name());
                        save_level_to_file::<V>(file.path(), &mut self.world)
                            .map(|_| println!("Level saved!"))
                            .unwrap_or_else(|_| println!("Could not save level."));
                    }

                    let mut input = self.world.write_resource::<Input>();
                    input.movement_mode = input.last_movement_mode;
                }
                _ => (),
            },
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => event_loop_proxy
                .send_event(CustomEvent::Quit)
                .unwrap_or_default(),
            Event::MainEventsCleared => {
                //display.gl_window().window().request_redraw();
                //if input.update {}
                if Instant::now() > fps_timer.start + Duration::from_millis(FRAME_MS) {
                    fps_timer.end();
                    {
                        self.world.write_resource::<Input>().frame_duration =
                            fps_timer.get_frame_length();
                    }
                    fps_timer.start();
                    self.on_redraw(display);
                }
            }
            Event::RedrawRequested(_) => {}
            _ => (),
        };
        // this catches e.g. all window resizing events
        self.update_gui(display, event, fps_timer);
    }

    fn update_gui<E>(&mut self, display: &Display, event: &Event<E>, fps_timer: &mut FPSTimer) {
        let gui_config = &self.world.fetch::<Config>().gui;
        let ui_args = match gui_config {
            crate::config::GuiConfig::None => UIArgs::None,
            crate::config::GuiConfig::Simple => UIArgs::Simple {
                frame_duration: fps_timer.get_frame_length(),
                coins_collected: self.world.read_resource::<crate::coin::CoinsCollected>().0,
                coins_left: self.world.read_storage::<crate::coin::Coin>().count() as u32,
            },
            crate::config::GuiConfig::Debug => {
                UIArgs::new_debug::<V>(&self.world, fps_timer.get_frame_length())
            }
        };

        //gui update (all events)
        if let Some(ref mut gui) = &mut self.gui {
            gui.update(display, &mut fps_timer.gui_last_time, event, ui_args)
        };
    }

    fn run_systems(&mut self) {
        self.dispatcher.dispatch(&self.world);
        self.world.maintain();
    }

    fn on_redraw(&mut self, display: &Display) {
        // Redraw the application.
        self.run_systems();

        self.draw(display);

        Self::center_mouse(&mut self.world.write_resource::<Input>(), display)
    }

    fn center_mouse(input: &mut Input, display: &Display) {
        if input.is_mouse_locked() {
            display
                .gl_window()
                .window()
                .set_cursor_position(glium::glutin::dpi::Position::new(
                    glium::glutin::dpi::PhysicalPosition::new(100, 100),
                ))
                .unwrap();
            display.gl_window().window().set_cursor_visible(false);
            input.mouse.mouse_dpos = (0., 0.);
        } else {
            display.gl_window().window().set_cursor_visible(true);
        }
    }

    fn draw(&mut self, display: &Display) {
        // TODO: ideally all this stuff is invisible to engine - it should be enough to pass lines
        // and gui args to graphics for drawing
        let draw_lines_data: ReadExpect<draw::DrawLineList<V::SubV>> = self.world.system_data();
        let draw_lines = &draw_lines_data.0;
        self.graphics.update_buffer(draw_lines, display);

        let mut target = display.draw();
        target = self.graphics.draw_lines(draw_lines, target);
        //draw gui
        if let Some(ref mut gui) = &mut self.gui {
            gui.draw(
                display,
                &mut target,
                &mut self.world.fetch_mut(),
                &self.world.fetch(),
            );
        }
        target.finish().unwrap();
    }
}

impl<V, G> EngineD<V, G>
where
    V: VectorTrait + Componentable + Serialize + DeserializeOwned,
    V::SubV: Componentable + Serialize + DeserializeOwned,
    V::M: Componentable + Serialize + DeserializeOwned,
    G: GraphicsTrait,
{
    fn init(config: &Config, display: &Display, gui: Option<crate::gui::GuiSystem>) -> Self {
        println!("Starting {}d engine", V::DIM);
        Self::new(config, G::init(display), gui)
    }
}

//this essentially turns EngineD into an enum
//could probably use a macro here
//there must be a nicer way
pub enum Engine {
    Three(EngineD<Vec3, DefaultGraphics>),
    Four(EngineD<Vec4, DefaultGraphics>),
}
impl Engine {
    pub fn init(
        dim: ValidDimension,
        config: &Config,
        display: &Display,
        gui: Option<crate::gui::GuiSystem>,
    ) -> Engine {
        match dim {
            ValidDimension::Three => Engine::Three(EngineD::<Vec3, _>::init(config, display, gui)),
            ValidDimension::Four => Engine::Four(EngineD::<Vec4, _>::init(config, display, gui)),
        }
    }

    pub fn new(dim: ValidDimension, config: &Config, display: &Display) -> Engine {
        let gui = Some(crate::gui::init("test", display));
        Self::init(dim, config, display, gui)
    }

    pub fn restart(&mut self, dim: ValidDimension, config: &Config, display: &Display) -> Engine {
        let mut gui = None;
        std::mem::swap(
            &mut gui,
            match self {
                Engine::Three(EngineD { gui, .. }) => gui,
                Engine::Four(EngineD { gui, .. }) => gui,
            },
        );
        Self::init(dim, config, display, gui)
    }
    pub fn update(
        &mut self,
        event: &Event<CustomEvent>,
        event_loop_proxy: &EventLoopProxy<CustomEvent>,
        display: &Display,
        fps_timer: &mut FPSTimer,
    ) {
        match self {
            Engine::Three(e) => e.update(event, event_loop_proxy, display, fps_timer),
            Engine::Four(e) => e.update(event, event_loop_proxy, display, fps_timer),
        }
    }
}
