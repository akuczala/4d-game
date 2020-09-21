
use std::time::{Duration,Instant};
use crate::FPSTimer;
use crate::spatial_hash::SpatialHashSet;
use crate::coin::Coin;
use crate::collide;
use specs::prelude::*;
use glium::Display;
use std::marker::PhantomData;

use glium::glutin::{
        event::{Event, WindowEvent},
        event_loop::ControlFlow,
    };

use crate::geometry::{Shape};

use crate::camera::Camera;
use crate::clipping::ClipState;
use crate::draw;
use crate::gui::UIArgs;
//NOTES:
// include visual indicator of what direction a collision is in

use crate::input::{Input,MovementMode};

use crate::graphics::{Graphics,Graphics2d,Graphics3d};
use crate::vector::{Vec3,Vec4,VecIndex,VectorTrait,Field};
use crate::fps::FPSFloat;



pub struct EngineD<V : VectorTrait, G : Graphics<V::SubV>> {
    pub world : World,
    pub cur_lines_length : usize,
    graphics : G,
    gui : Option<crate::gui::System>,
    dispatcher : Dispatcher<'static,'static>,
    dummy : PhantomData<V>, //Forces EngineD to take V as a parameter
}
impl<V : VectorTrait, G : Graphics<V::SubV>> EngineD<V,G>
{
    pub fn new<F : Fn(&mut World)>(build_scene : F, graphics : G, display : &Display, maybe_gui : Option<crate::gui::System>) -> Self {

        let mut world = World::new();

        let mut dispatcher = DispatcherBuilder::new()
            .with(crate::input::UpdateCameraSystem(PhantomData::<V>),"update_camera",&[])
            .with(crate::collide::PlayerCollisionDetectionSystem(PhantomData::<V>),"player_collision_detect",&["update_camera"]) 
            .with(crate::collide::PlayerStaticCollisionSystem(PhantomData::<V>),"player_static_collision",&["player_collision_detect"])
            .with(crate::collide::PlayerCoinCollisionSystem(PhantomData::<V>),"player_coin_collision",&["player_collision_detect","player_static_collision"])
            .with(crate::collide::MovePlayerSystem(PhantomData::<V>),"move_player",&["player_static_collision","player_coin_collision"])
            .with(crate::collide::UpdatePlayerBBox(PhantomData::<V>),"update_player_bbox",&["move_player"]) //merge with above
            .with(crate::coin::CoinSpinningSystem(PhantomData::<V>),"coin_spinning",&[])
            .with(crate::input::PrintDebugSystem(PhantomData::<V>),"print_debug",&["update_camera"])
            //start drawing phase
            //for each shape, update clipping boundaries and face visibility
            .with(draw::VisibilitySystem(PhantomData::<V>),"visibility",&["coin_spinning","move_player"])
            //determine what shapes are in front of other shapes
            .with(crate::clipping::InFrontSystem(PhantomData::<V>),"in_front",&["visibility"])
            //calculate and clip lines for each shape
            .with(draw::CalcShapesLinesSystem(PhantomData::<V>),"calc_shapes_lines",&["in_front"])
            //project lines
            .with(draw::TransformDrawLinesSystem(PhantomData::<V>),"transform_draw_lines",&["calc_shapes_lines"])
            .build();

        dispatcher.setup(&mut world);

        world.insert(Input::new());

        build_scene(&mut world);
        let shapes_len = world.read_component::<Shape<V>>().count();

        collide::create_spatial_hash::<V>(&mut world);


        //use crate::vector::Rotatable;
        //camera.rotate(-2,-1,3.14159/2.);
        
        let clip_state = ClipState::<V>::new(shapes_len);
        let draw_lines = draw::DrawLineList::<V>(vec![]);
        let proj_lines = draw_lines.map(|l| draw::transform_draw_line(l,&Camera::new(V::zero()))); // <-- dummy camera
         //draw_lines.append(&mut crate::draw::draw_wireframe(&test_cube,GREEN));
        let cur_lines_length = draw_lines.len();
        let face_scales : Vec<crate::vector::Field> = vec![0.9];

        
        //DEBUG
        // {
        // let entities = world.read_resource::<Entities>();
        // entities.delete(entities.entity(20)).unwrap();
        // }

        world.insert(clip_state); // decompose into single entity properties
        world.insert(draw_lines); // unclear if this would be better as entities
        world.insert(proj_lines);
        world.insert(face_scales);
        
        EngineD {
            world,
            dispatcher,
            cur_lines_length,
            graphics,
            gui : maybe_gui,
            dummy : PhantomData,
        }
    }

    //currently returns bool that tells main whether to swap engines
    //runs for each event
    pub fn update<E>(&mut self, event : &Event<E>, control_flow : &mut ControlFlow, display : &Display, fps_timer : &mut FPSTimer) -> bool {
        let frame_duration = fps_timer.get_frame_length();
        let mut ui_args = UIArgs::None;
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

            //values to be passed to UI
            ui_args = UIArgs::Test{
                frame_duration,
                elapsed_time : fps_timer.elapsed_time,
                mouse_diff : input.mouse_dpos,
                mouse_pos : input.helper.mouse()
            };

            if input.closed {
                println!("Escape button pressed; exiting.");
                *control_flow = ControlFlow::Exit;
            }
            //window / game / redraw events
            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    println!("The close button was pressed; stopping");
                    *control_flow = ControlFlow::Exit
                },
                Event::MainEventsCleared => {
                    // Application update code.
                    display.gl_window().window().request_redraw();
                    if input.update {
                        
                        
                    }
                },
                _ => (),
            };
        }
        //gui update (all events)
        if let Some(ref mut gui) = &mut self.gui {
            gui.update(&display, &mut fps_timer.gui_last_time, &event, control_flow, ui_args)
        };
        //game update and draw
        match event {
            Event::MainEventsCleared => {},
            //frame update
            Event::RedrawRequested(_) => {
                // Redraw the application.
                if Instant::now() > fps_timer.start + Duration::from_millis(16) {
                    fps_timer.end();
                    {
                        self.world.write_resource::<Input>().frame_duration = fps_timer.get_frame_length();
                    }
                    fps_timer.start();

                    self.dispatcher.dispatch(&mut self.world);
                    self.world.maintain();

                    self.draw(&display);

                    {
                        let mut input = self.world.write_resource::<Input>();
                        if let MovementMode::Mouse = input.movement_mode {
                            display.gl_window().window().set_cursor_position(glium::glutin::dpi::Position::new(glium::glutin::dpi::PhysicalPosition::new(100,100))).unwrap();
                            display.gl_window().window().set_cursor_visible(false);
                            input.mouse_dpos = (0.,0.);
                        } else {
                            display.gl_window().window().set_cursor_visible(true);
                        }
                    }

                }
            },
            _ => ()
        };

        false //don't switch engines
    }

    fn draw(&mut self, display : &Display) {
        
        let draw_lines_data : ReadExpect<draw::DrawLineList<V::SubV>> = self.world.system_data();
        let draw_lines = &(&draw_lines_data).0;
        //make new buffer if the number of lines changes
        if draw_lines.len() != self.cur_lines_length {
            self.graphics.new_vertex_buffer_from_lines(draw_lines,display);
            //println!("New buffer! {} to {}",draw_lines.len(),cur_lines_length);
            self.cur_lines_length = draw_lines.len();
        }

        let mut target = display.draw();
        target = self.graphics.draw_lines(&draw_lines,target);
        //draw gui
        if let Some(ref mut gui) = &mut self.gui {
            gui.draw(&display, &mut target);
        }
        target.finish().unwrap();
    }

}

impl EngineD<Vec3,Graphics2d> {
    fn init(display : &Display, gui : Option<crate::gui::System>) -> Self {
        println!("Starting 3d engine");
        //let game = Game::new(game::build_shapes_3d());
        let mut graphics = Graphics2d::new(display);
        graphics.new_vertex_buffer_from_lines(&vec![],display);

        Self::new(crate::build_level::build_shapes_3d,graphics,display,gui)
    }
}
impl EngineD<Vec4,Graphics3d> {
    fn init(display : &Display, gui : Option<crate::gui::System>) -> Self {
        println!("Starting 4d engine");
        //let game = Game::new(game::build_shapes_4d());
        let mut graphics = Graphics3d::new(display);
        graphics.new_vertex_buffer_from_lines(&vec![],display);
        Self::new(crate::build_level::build_shapes_4d,graphics,display,gui)
    }
}

//this essentially turns EngineD into an enum
//could probably use a macro here
//there must be a nicer way
pub enum Engine {
    Three(EngineD<Vec3,Graphics2d>),
    Four(EngineD<Vec4,Graphics3d>)
}
impl Engine {
    pub fn init(dim : VecIndex, display : & Display) -> Engine {
        let gui = Some(crate::gui::init(&"test",&display));
        //let gui = None;
        match dim {
            3 => Ok(Engine::Three(EngineD::<Vec3,Graphics2d>::init(display,gui))),
            4 => Ok(Engine::Four(EngineD::<Vec4,Graphics3d>::init(display,gui))),
            _ => Err("Invalid dimension for game engine")
        }.unwrap()
    }
    pub fn swap_dim(&mut self, dim : VecIndex, display : &Display) -> Engine {
        let mut gui : Option<crate::gui::System> = None;
        match self {
            Engine::Four(engined) => std::mem::swap(&mut gui, &mut engined.gui),
            Engine::Three(engined) => std::mem::swap(&mut gui, &mut engined.gui)
        }
        match self {
            Engine::Four(_engined) => Engine::Three(EngineD::<Vec3,Graphics2d>::init(display,gui)),
            Engine::Three(_engined) => Engine::Four(EngineD::<Vec4,Graphics3d>::init(display,gui)),
        }
    }
    pub fn update<E>(&mut self, event : &Event<E>, control_flow : &mut ControlFlow, display : &Display, fps_timer : &mut FPSTimer) -> bool{
        match self {
                    Engine::Three(e) => e.update(event,control_flow,display,fps_timer),
                    Engine::Four(e) => e.update(event,control_flow,display,fps_timer),
                }
    }

}

