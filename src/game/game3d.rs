use crate::geometry::{Shape,Line};
use crate::vector::{VectorTrait,Vec3};
use crate::build_level;
use super::Game;
use crate::draw::Camera;
use crate::clipping::ClipState;
pub struct Game3d {
    pub shapes : Vec<Shape<Vec3>>,
    pub extra_lines : Vec<Line<Vec3>>,
    pub camera : Camera<Vec3>,
    pub clip_state : ClipState<Vec3>
}

impl Game<Vec3> for Game3d {

	fn new() -> Self {
		//let graphics = crate::graphics::Graphics2d::new(display);
		let shapes = build_shapes_3d();
	    //let mut extra_lines = buildfloor::build_floor3(5,1.0,0.0);
	    //extra_lines.append(&mut buildfloor::build_floor3(5,1.0,1.0));
	    let extra_lines : Vec<Line<Vec3>> = Vec::new();
	    //let extra_lines = draw::Texture::make_tile_texture(&vec![0.3,0.8],&vec![3,4,5]);
	    let camera = Camera::new(Vec3::new(0.0,0.0,0.0));
	    //camera.look_at(shapes[0].get_pos());
	    let mut newGame = Game3d {
	    	shapes,
	    	extra_lines,
	    	camera,
            clip_state : ClipState::new(&vec![])
	    };
	    newGame.clip_state = ClipState::new(&newGame.shapes);
	    newGame

	}
	fn get_clip_state(&mut self) -> &mut ClipState<Vec3> {
        &mut self.clip_state
    }
    fn get_camera(&mut self) -> &mut Camera<Vec3> { 
        &mut self.camera
    }
    fn get_shapes(&mut self) -> &mut Vec<Shape<Vec3>> { 
        &mut self.shapes
    }
    fn get_extra_lines(&mut self) -> &mut Vec<Line<Vec3>> {
        &mut self.extra_lines
    }
}

pub fn build_shapes_3d() -> Vec<Shape<Vec3>> {

    build_level::build_lvl_1_3d()
    //build_level::build_test_scene_3d()
}

// pub fn game_3d(input : &'static mut Input, display : & glium::Display, events_loop : &'static EventLoop<()>) {
    
    
    

    
//     game(graphics,input,events_loop,shapes,camera,extra_lines);
// }