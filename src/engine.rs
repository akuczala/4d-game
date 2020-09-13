//can probably merge with Game?
//or mix and match responsibilities

use glium::Display;

//NOTES:
// include visual indicator of what direction a collision is in

use crate::input::Input;
use crate::game::Game;
use crate::game;
use crate::graphics::{Graphics,Graphics2d,Graphics3d};
use crate::vector::{Vec3,Vec4,VecIndex,VectorTrait};
use crate::fps::FPSFloat;

pub struct EngineD<V : VectorTrait,G : Graphics<V::SubV>> {
    game : Game<V>,
    graphics : G
}
impl<V,G> EngineD<V,G>
where V : VectorTrait, G : Graphics<V::SubV> {
    fn game_update(&mut self, input : &mut Input, frame_len : FPSFloat) {
        self.game.game_update(input, frame_len);
    }
    fn draw(&mut self, display : &Display) {
        self.game.draw_update(&mut self.graphics,display);
    }
    fn print_debug(&mut self, input : &mut Input, frame_len : FPSFloat) {
       input.print_debug(&mut self.game, frame_len);
    }
}
impl EngineD<Vec3,Graphics2d> {
    fn init(display : &Display) -> Self {
        println!("Starting 3d engine");
        let game = Game::new(game::build_shapes_3d());
        let mut graphics = Graphics2d::new(display);
        graphics.new_vertex_buffer_from_lines(&vec![],display);

        Self{game, graphics}
    }
}
impl EngineD<Vec4,Graphics3d> {
    fn init(display : &Display) -> Self {
        println!("Starting 4d engine");
        let game = Game::new(game::build_shapes_4d());
        let mut graphics = Graphics3d::new(display);
        graphics.new_vertex_buffer_from_lines(&vec![],display);

        Self{game, graphics}
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
    pub fn game_update(&mut self, input : &mut Input, frame_len : FPSFloat) {
        match self {
                    Engine::Three(e) => e.game_update(input, frame_len),
                    Engine::Four(e) => e.game_update(input, frame_len),
                }
    }
    pub fn draw(&mut self, display : &Display) {
        match self {
            Engine::Three(e) => e.draw(display),
            Engine::Four(e) => e.draw(display)
        };
    }
    pub fn print_debug(&mut self, input : &mut Input, frame_len : FPSFloat) {
        match self {
            Engine::Three(e) => e.print_debug(input, frame_len),
            Engine::Four(e) => e.print_debug(input, frame_len)
        };
    }

}