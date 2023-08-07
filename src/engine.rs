mod dispatcher;
pub use dispatcher::get_engine_dispatcher_builder;
use serde::de::DeserializeOwned;

use crate::collide;
use crate::config::load_config;
use crate::constants::FRAME_MS;
use crate::ecs_utils::Componentable;
use crate::graphics::DefaultGraphics;
use crate::graphics::GraphicsTrait;
use crate::input::ShapeManipulationState;
use crate::vector::MatrixTrait;
use crate::FPSTimer;
use glium::Display;
use specs::prelude::*;
use std::marker::PhantomData;
use std::time::{Duration, Instant};

use crate::draw;
use crate::gui::UIArgs;
use glium::glutin::{
    event::{Event, WindowEvent},
    event_loop::ControlFlow,
};
//NOTES:
// include visual indicator of what direction a collision is in

use crate::input::{Input, MovementMode, PlayerMovementMode};

use crate::components::*;
use crate::geometry::shape::RefShapes;
use crate::vector::{Vec3, Vec4, VecIndex, VectorTrait};

// TODO: reduce number of explicit constraints needed by introducing a componentable-constrained trait?
pub struct EngineD<V, G> {
    pub world: World,
    graphics: G,
    gui: Option<crate::gui::System>,
    dispatcher: Dispatcher<'static, 'static>,
    dummy: PhantomData<V>, //Forces EngineD to take V as a parameter
}
impl<V, G> EngineD<V, G>
where
    V: VectorTrait + Componentable,
    V::SubV: Componentable,
    V::M: Componentable,
    G: GraphicsTrait,
{
    pub fn new<F: Fn(&mut World)>(
        build_scene: F,
        graphics: G,
        maybe_gui: Option<crate::gui::System>,
    ) -> Self {
        let mut world = World::new();

        let mut dispatcher = get_engine_dispatcher_builder::<V>().build();

        dispatcher.setup(&mut world);

        world.insert(Input::new());
        world.insert(ShapeManipulationState::default() as ShapeManipulationState<V, V::M>);
        world.insert(load_config());

        build_scene(&mut world);

        collide::create_spatial_hash::<V>(&mut world);

        let clip_state = ClipState::<V>::new();
        let draw_lines: DrawLineList<V> = draw::DrawLineList::<V>(vec![]);
        let proj_lines = DrawLineList::<V::SubV>(vec![]);

        world.insert(clip_state); // decompose into single entity properties
        world.insert(draw_lines); // unclear if this would be better as entities; might be able to thread
        world.insert(proj_lines);

        EngineD {
            world,
            dispatcher,
            graphics,
            gui: maybe_gui,
            dummy: PhantomData,
        }
    }

    //currently returns bool that tells main whether to swap engines
    //runs for each event
    pub fn update<E>(
        &mut self,
        event: &Event<E>,
        control_flow: &mut ControlFlow,
        display: &Display,
        fps_timer: &mut FPSTimer,
    ) -> bool {
        //brackets here used to prevent borrowing issues between input + self
        //would probably be sensible to move this into its own function
        {
            let mut input = self.world.write_resource::<Input>();
            //swap / reset engine
            if input.swap_engine {
                return true;
            }
            //input events
            input.listen_events(event);
            if input.closed {
                println!("Escape button pressed; exiting.");
                *control_flow = ControlFlow::Exit;
            }
        }
        //window / game / redraw events
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                println!("The close button was pressed; stopping");
                *control_flow = ControlFlow::Exit
            }
            Event::MainEventsCleared => {
                // Application update code.
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
        self.update_ui(display, control_flow, event, fps_timer);

        false //don't switch engines
    }
    fn update_ui<E>(
        &mut self,
        display: &Display,
        control_flow: &mut ControlFlow,
        event: &Event<E>,
        fps_timer: &mut FPSTimer,
    ) {
        //let frame_duration = fps_timer.get_frame_length();
        //let input: &Input = &self.world.read_resource::<Input>();
        //values to be passed to UI
        // let ui_args = UIArgs::Test{
        //     frame_duration,
        //     elapsed_time : fps_timer.elapsed_time,
        //     mouse_diff : input.mouse.mouse_dpos,
        //     mouse_pos : input.helper.mouse()
        // };

        //TODO: slow to build this
        let ui_args = UIArgs::new_debug::<V>(&self.world, fps_timer.get_frame_length());

        //let ui_args = UIArgs::None;
        // let ui_args = UIArgs::Simple {
        //     frame_duration: fps_timer.get_frame_length(),
        //     coins_collected: self.world.read_resource::<crate::coin::CoinsCollected>().0,
        //     coins_left: self.world.read_storage::<crate::coin::Coin>().count() as u32,
        // };
        //gui update (all events)
        if let Some(ref mut gui) = &mut self.gui {
            gui.update(
                display,
                &mut fps_timer.gui_last_time,
                event,
                control_flow,
                ui_args,
            )
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
        if let MovementMode::Player(PlayerMovementMode::Mouse) | MovementMode::Shape(_) =
            input.movement_mode
        {
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
            gui.draw(display, &mut target);
        }
        target.finish().unwrap();
    }
}

impl<V, G> EngineD<V, G>
where
    V: VectorTrait + Componentable + DeserializeOwned,
    V::SubV: Componentable + DeserializeOwned,
    V::M: Componentable + DeserializeOwned,
    G: GraphicsTrait,
{
    fn init(display: &Display, gui: Option<crate::gui::System>) -> Self {
        println!("Starting {}d engine", V::DIM);
        Self::new(crate::build_level::build_scene::<V>, G::init(display), gui)
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
    pub fn init(dim: VecIndex, display: &Display) -> Engine {
        let gui = Some(crate::gui::init("test", display));
        //let gui = None;
        match dim {
            3 => Ok(Engine::Three(EngineD::<Vec3, DefaultGraphics>::init(
                display, gui,
            ))),
            4 => Ok(Engine::Four(EngineD::<Vec4, DefaultGraphics>::init(
                display, gui,
            ))),
            _ => Err("Invalid dimension for game engine"),
        }
        .unwrap()
    }
    pub fn swap_dim(&mut self, display: &Display) -> Engine {
        let mut gui: Option<crate::gui::System> = None;
        match self {
            Engine::Four(engined) => std::mem::swap(&mut gui, &mut engined.gui),
            Engine::Three(engined) => std::mem::swap(&mut gui, &mut engined.gui),
        }
        match self {
            Engine::Four(_engined) => {
                Engine::Three(EngineD::<Vec3, DefaultGraphics>::init(display, gui))
            }
            Engine::Three(_engined) => {
                Engine::Four(EngineD::<Vec4, DefaultGraphics>::init(display, gui))
            }
        }
    }
    pub fn update<E>(
        &mut self,
        event: &Event<E>,
        control_flow: &mut ControlFlow,
        display: &Display,
        fps_timer: &mut FPSTimer,
    ) -> bool {
        match self {
            Engine::Three(e) => e.update(event, control_flow, display, fps_timer),
            Engine::Four(e) => e.update(event, control_flow, display, fps_timer),
        }
    }
}
