//can probably merge with Game?
//or mix and match responsibilities
use specs::prelude::*;
use glium::Display;
use std::marker::PhantomData;

use crate::geometry::{Shape,Line};

use crate::camera::Camera;
use crate::clipping::ClipState;
use crate::draw;
//NOTES:
// include visual indicator of what direction a collision is in

use crate::input::Input;
//use crate::game::Game;
//use crate::game;
use crate::graphics::{Graphics,Graphics2d,Graphics3d};
use crate::vector::{Vec3,Vec4,VecIndex,VectorTrait};
use crate::fps::FPSFloat;

pub struct EngineD<V : VectorTrait,G : Graphics<V::SubV>> {
    pub world : World,
    //pub extra_lines : Vec<Line<V>>,
    pub cur_lines_length : usize,
    graphics : G,
    dummy : PhantomData<V>,
}
impl<V,G> EngineD<V,G>
where V : VectorTrait, G : Graphics<V::SubV> {
    pub fn new(shapes : Vec<Shape<V>>, graphics : G) -> Self
    {
        let mut world = World::new();
        world.register::<Shape<V>>();

        //change to into_iter, remove cloning
        let shapes_len = shapes.len();
        for shape in shapes.into_iter() {
            world.create_entity().with(shape).build();
        }

        //let extra_lines : Vec<Line<V>> = Vec::new();
        let camera = Camera::new(V::zero()-V::one_hot(-1)*0.);
        //use crate::vector::Rotatable;
        //camera.rotate(-2,-1,3.14159/2.);
        
        let clip_state = ClipState::<V>::new(shapes_len);
        let draw_lines = draw::DrawLineList::<V>(vec![]);
        let proj_lines = draw_lines.map(|l| draw::transform_draw_line(l,&camera));
         //draw_lines.append(&mut crate::draw::draw_wireframe(&test_cube,GREEN));
        let cur_lines_length = draw_lines.len();
        let face_scales : Vec<crate::vector::Field> = vec![0.8,0.9];

        world.insert(clip_state);
        world.insert(draw_lines);
        world.insert(proj_lines);
        world.insert(face_scales);
        world.insert(camera);
        EngineD {
            world,
            cur_lines_length,
            graphics,
            dummy : PhantomData
        }
    }
    //temporary functions to accomodate non-ecs code
    //required changing function arguments from Vec to slice
    pub fn get_shapes<'a>(&'a self) -> ReadStorage<'a,Shape<V>> {
        //let data : ReadStorage<Shape<V>> = self.world.system_data();
        return self.world.system_data()
    }
    pub fn get_mut_shapes(&mut self) -> WriteStorage<Shape<V>> {
       self.world.system_data()
    }
    //game update every frame
    fn game_update(&mut self, input : &mut Input) {
        //self.game.game_update(input);


        if input.update {
            //if input.pressed.being_touched {
            if true {
                let shapes_len = self.get_shapes().as_slice().len();
                //currently rotates the coin (happens to be the last entity with Shape)
                self.get_mut_shapes().as_mut_slice()[shapes_len-1].rotate(0,-1,0.05);

            }
            //let mut dispatcher = DispatcherBuilder::new()
            //    .with(crate::input::UpdateCameraSystem(V::zero()),"update_camera",&[])
            //    .build();

            //dispatcher.dispatch(&mut self.world);
            //update_camera(input, &mut self.camera, frame_len);

        let shapes_len = self.get_shapes().as_slice().len();
        crate::input::update_shape(input, &mut self.get_mut_shapes().as_mut_slice()[shapes_len-1]);
        
        //input.print_debug(&self.camera,&game_duration,&frame_duration,&mut clip_state,&shapes);
            input.update = true; //set to true for constant updating
        }

    }

    fn draw(&mut self, display : &Display) {
        //self.game.draw_update(&mut self.graphics,display);
        //self.draw_lines = self.draw_stuff();

        //V::zero() tells the system what dimension it's working in.
        //would like to do this with phantom data or something
        //would ideally define dispatcher on init
        let mut dispatcher = DispatcherBuilder::new()
                //for each shape, update clipping boundaries and face visibility
                .with(draw::VisibilitySystem(V::zero()),"visibility",&[])
                //determine what shapes are in front of other shapes
                .with(crate::clipping::InFrontSystem(V::zero()),"in_front",&["visibility"])
                //calculate and clip lines for each shape
                .with(draw::CalcShapesLinesSystem(V::zero()),"calc_shapes_lines",&["visibility","in_front"])
                //project lines
                .with(draw::TransformDrawLinesSystem(V::zero()),"transform_draw_lines",&["visibility","in_front","calc_shapes_lines"])
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

    fn print_debug(&mut self, input : &mut Input) {
       crate::input::print_debug::<V>(input);
    }
    
}
impl EngineD<Vec3,Graphics2d> {
    fn init(display : &Display) -> Self {
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
            3 => Ok(Engine::Three(EngineD::<Vec3,Graphics2d>::init(&display))),
            4 => Ok(Engine::Four(EngineD::<Vec4,Graphics3d>::init(&display))),
            _ => Err("Invalid dimension for game engine")
        }.unwrap()
    }
    pub fn game_update(&mut self, input : &mut Input) {
        match self {
                    Engine::Three(e) => e.game_update(input),
                    Engine::Four(e) => e.game_update(input),
                }
    }
    pub fn draw(&mut self, display : &Display) {
        match self {
            Engine::Three(e) => e.draw(display),
            Engine::Four(e) => e.draw(display)
        };
    }
    pub fn print_debug(&mut self, input : &mut Input) {
        match self {
            Engine::Three(e) => e.print_debug(input),
            Engine::Four(e) => e.print_debug(input)
        };
    }

}

