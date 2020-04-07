use crate::geometry::{Shape,Line};
use crate::geometry::buildshapes;
use crate::vector::{VectorTrait,Vec4};
use crate::colors::*;
use crate::build_level;
use super::Game;
use crate::draw::Camera;
use crate::clipping::ClipState;
pub struct Game4d {
    pub shapes : Vec<Shape<Vec4>>,
    pub extra_lines : Vec<Line<Vec4>>,
    pub camera : Camera<Vec4>,
    pub clip_state : ClipState<Vec4>,
}

impl Game<Vec4> for Game4d {

	fn new() -> Self {
		//let graphics = crate::graphics::Graphics3d::new(display);
    let shapes = build_shapes_4d();

    let extra_lines : Vec<Line<Vec4>> = Vec::new();
    let camera = Camera::new(Vec4::new(0.0,0.0,0.0,0.0));
	let mut newGame = Game4d {
	    	shapes,
	    	extra_lines,
	    	camera,
            clip_state : ClipState::new(&vec![])
	    };
    newGame.clip_state = ClipState::new(&newGame.shapes);
    newGame
	}
    fn get_clip_state(&mut self) -> &mut ClipState<Vec4> {
        &mut self.clip_state
    }
    fn get_camera(&mut self) -> &mut Camera<Vec4> { 
        &mut self.camera
    }
    fn get_shapes(&mut self) -> &mut Vec<Shape<Vec4>> { 
        &mut self.shapes
    }
    fn get_extra_lines(&mut self) -> &mut Vec<Line<Vec4>> {
        &mut self.extra_lines
    }

}

pub fn build_shapes_4d() -> Vec<Shape<Vec4>> {
    let wall_length = 3.0;
    //buildshapes::build_axes_cubes_4d()
    //buildshapes::cubeidor_4d()
    let mut shapes = build_level::build_corridor_cross(
        &buildshapes::color_cube(buildshapes::build_cube_4d(1.0)),wall_length);
    //let (m,n) = (4,4);
    //let mut duocylinder = buildshapes::build_duoprism_4d([1.0,1.0],[[0,1],[2,3]],[m,n])
    shapes.push(buildshapes::build_duoprism_4d([0.1,0.1],[[0,1],[2,3]],[6,6])
        .set_color(YELLOW)
        .set_pos(&Vec4::new(0.0,0.0,wall_length - 0.5,0.0)));
    //let shapes_len = shapes.len();
    //buildshapes::color_duocylinder(&mut shapes[shapes_len-1],10,10);
    shapes
     //   .set_pos(&Vec4::new(0.0,0.0,0.0,0.0));
    
}
// pub fn game_4d(input : &'static mut Input, display : & glium::Display, events_loop : &'static EventLoop<()>) {

    
//     //camera.look_at(shapes[0].get_pos());
//     game(graphics,input,events_loop,shapes,camera,extra_lines);
// }
