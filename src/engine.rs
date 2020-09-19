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
//NOTES:
// include visual indicator of what direction a collision is in

use crate::input::Input;

use crate::graphics::{Graphics,Graphics2d,Graphics3d};
use crate::vector::{Vec3,Vec4,VecIndex,VectorTrait,Field};
use crate::fps::FPSFloat;

pub struct Player(pub Entity); //specifies entity of the player?

pub struct EngineD<V : VectorTrait, G : Graphics<V::SubV>> {
    pub world : World,
    //pub extra_lines : Vec<Line<V>>,
    pub cur_lines_length : usize,
    graphics : G,
    //Forces EngineD to take V as a parameter
    dummy : PhantomData<V>,
}
impl<V : VectorTrait, G : Graphics<V::SubV>> EngineD<V,G>
{
    pub fn new(mut shapes : Vec<Shape<V>>, graphics : G) -> Self
    {
        let mut world = World::new();
        world.register::<Shape<V>>();
        world.register::<crate::coin::Coin>();
        world.register::<Camera<V>>();
        world.register::<BBox<V>>();
        world.register::<collide::StaticCollider>();
        world.register::<collide::MoveNext<V>>();

        world.insert(Input::new());

        //add shape entities and intialize spatial hash set
        let shapes_len = shapes.len();
        let coin_shape = shapes.pop();
        let (mut max, mut min) = (V::zero(), V::zero());
        let mut max_lengths = V::zero();

        for shape in shapes.into_iter() {
            let bbox = collide::calc_bbox(&shape);
            min = min.zip_map(bbox.min,Field::min); 
            max = max.zip_map(bbox.max,Field::max);
            max_lengths = max_lengths.zip_map(bbox.max - bbox.min,Field::max);
            world.create_entity()
            .with(bbox)
            .with(shape)
            .with(collide::StaticCollider)
            .build();
        }
        //println!("Min/max: {},{}",min,max);
        //println!("Longest sides {}",max_lengths);
        world.insert(
            SpatialHashSet::<V,Entity>::new(
                min*1.5, //make bounds slightly larger than farthest points
                max*1.5,
                max_lengths*1.1 //make cell size slightly larger than largest shape dimensions
            )
        );
        //enter shapes into hash set
        collide::BBoxHashingSystem(PhantomData::<V>).run_now(&world);

        world.create_entity()
            .with(coin_shape.unwrap())
            .with(Coin)
            .build();

        //let extra_lines : Vec<Line<V>> = Vec::new();
        let camera = Camera::new(V::zero());


        //use crate::vector::Rotatable;
        //camera.rotate(-2,-1,3.14159/2.);
        
        let clip_state = ClipState::<V>::new(shapes_len);
        let draw_lines = draw::DrawLineList::<V>(vec![]);
        let proj_lines = draw_lines.map(|l| draw::transform_draw_line(l,&camera));
         //draw_lines.append(&mut crate::draw::draw_wireframe(&test_cube,GREEN));
        let cur_lines_length = draw_lines.len();
        let face_scales : Vec<crate::vector::Field> = vec![0.9];

        use crate::collide::BBox;
        let player_entity = world.create_entity()
            .with(BBox{min : V::ones()*(-0.1) + camera.pos, max : V::ones()*(0.1) + camera.pos})
            .with(camera) //decompose
            .with(collide::MoveNext::<V>::default())
            .build(); 

        world.insert(Player(player_entity));
        world.insert(clip_state); // decompose into single entity properties
        world.insert(draw_lines); // unclear if this would be better as entities
        world.insert(proj_lines);
        world.insert(face_scales);
        
        EngineD {
            world,
            cur_lines_length,
            graphics,
            dummy : PhantomData
        }
    }

    //currently returns bool that tells main whether to swap engines
    pub fn update<E>(&mut self, event : &Event<E>, control_flow : &mut ControlFlow, display : &Display, frame_duration : FPSFloat) -> bool {
        {
            self.world.write_resource::<Input>().frame_duration = frame_duration;
        }
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
                    if input.update {
                        display.gl_window().window().request_redraw();
                    }
                },
                _ => (),
            };
        }
        //game update and draw
        match event {
            Event::MainEventsCleared => {self.game_update()},
            Event::RedrawRequested(_) => {
                // Redraw the application.
                self.draw(&display);
                
            },
            _ => ()
        };

        false //don't switch engines
    }

    //game update every frame
    fn game_update(&mut self) {

        let mut dispatcher = DispatcherBuilder::new()
           .with(crate::input::UpdateCameraSystem(PhantomData::<V>),"update_camera",&[])
           .with(crate::collide::PlayerCollisionDetectionSystem(PhantomData::<V>),"player_collision",&["update_camera"])
           .with(crate::collide::MovePlayerSystem(PhantomData::<V>),"move_player",&["player_collision"])
           .with(crate::collide::UpdatePlayerBBox(PhantomData::<V>),"update_player_bbox",&["move_player"]) //merge with above
           .with(crate::coin::CoinSpinningSystem(PhantomData::<V>),"coin_spinning",&[])
           .with(crate::input::PrintDebugSystem(PhantomData::<V>),"print_debug",&["update_camera"])
           //.with(crate::collide::CollisionTestSystem(PhantomData::<V>),"collision_test",&["update_player_bbox"])
           .build();

        dispatcher.dispatch(&mut self.world);
    }

    fn draw(&mut self, display : &Display) {

        //would ideally define dispatcher on init
        let mut dispatcher = DispatcherBuilder::new()
                //for each shape, update clipping boundaries and face visibility
                .with(draw::VisibilitySystem(PhantomData::<V>),"visibility",&[])
                //determine what shapes are in front of other shapes
                .with(crate::clipping::InFrontSystem(PhantomData::<V>),"in_front",&["visibility"])
                //calculate and clip lines for each shape
                .with(draw::CalcShapesLinesSystem(PhantomData::<V>),"calc_shapes_lines",&["in_front"])
                //project lines
                .with(draw::TransformDrawLinesSystem(PhantomData::<V>),"transform_draw_lines",&["calc_shapes_lines"])
                .build();

        dispatcher.dispatch(&mut self.world);

        let draw_lines_data : ReadExpect<draw::DrawLineList<V::SubV>> = self.world.system_data();
        let draw_lines = &(&draw_lines_data).0;
        //make new buffer if the number of lines changes
        if draw_lines.len() != self.cur_lines_length {
            self.graphics.new_vertex_buffer_from_lines(draw_lines,display);
            //println!("New buffer! {} to {}",draw_lines.len(),cur_lines_length);
            self.cur_lines_length = draw_lines.len();
        }
        self.graphics.draw_lines(&draw_lines,display);
    }

}

impl EngineD<Vec3,Graphics2d> {
    fn init(display : &Display,) -> Self {
        println!("Starting 3d engine");
        //let game = Game::new(game::build_shapes_3d());
        let mut graphics = Graphics2d::new(display);
        graphics.new_vertex_buffer_from_lines(&vec![],display);

        Self::new(crate::build_level::build_shapes_3d(),graphics)
    }
}
impl EngineD<Vec4,Graphics3d> {
    fn init(display : &Display) -> Self {
        println!("Starting 4d engine");
        //let game = Game::new(game::build_shapes_4d());
        let mut graphics = Graphics3d::new(display);
        graphics.new_vertex_buffer_from_lines(&vec![],display);

        Self::new(crate::build_level::build_shapes_4d(),graphics)
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
    pub fn init(dim : VecIndex, display : &Display) -> Engine {
        match dim {
            3 => Ok(Engine::Three(EngineD::<Vec3,Graphics2d>::init(display))),
            4 => Ok(Engine::Four(EngineD::<Vec4,Graphics3d>::init(display))),
            _ => Err("Invalid dimension for game engine")
        }.unwrap()
    }
    pub fn update<E>(&mut self, event : &Event<E>, control_flow : &mut ControlFlow, display : &Display, frame_duration : FPSFloat) -> bool{
        match self {
                    Engine::Three(e) => e.update(event,control_flow,display,frame_duration),
                    Engine::Four(e) => e.update(event,control_flow,display,frame_duration),
                }
    }

}

