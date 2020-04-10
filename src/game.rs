use crate::geometry::{Shape,Line};
use crate::geometry::buildshapes;
use crate::vector::{VectorTrait,Vec3,Vec4};
use crate::colors::*;
use crate::build_level;
use crate::draw::Camera;
use crate::clipping::ClipState;
use crate::input::Input;
use crate::graphics::Graphics;
use crate::draw;

pub struct Game<V : VectorTrait> {
    pub shapes : Vec<Shape<V>>,
    pub extra_lines : Vec<Line<V>>,
    pub camera : Camera<V>,
    pub clip_state : ClipState<V>,
    pub draw_lines : Vec<Option<draw::DrawLine<V::SubV>>>,
    pub cur_lines_length : usize
}

impl<V> Game<V>
where V: VectorTrait
{
    pub fn new<'a,G>(shapes : Vec<Shape<V>>, graphics : &mut G) -> Self
    where G: Graphics<'a,V::SubV>
    {
        //let graphics = crate::graphics::Graphics3d::new(display);
        
        //let shapes : Vec<Shape<V>> = vec![];

        let extra_lines : Vec<Line<V>> = Vec::new();
        let camera = Camera::new(V::zero());
        let mut new_game = Game {
                shapes,
                extra_lines,
                camera,
                clip_state : ClipState::new(&vec![]),
                draw_lines : vec![],
                cur_lines_length : 0
            };
        new_game.clip_state = ClipState::new(&new_game.shapes);
        new_game.draw_lines = new_game.draw_stuff();
        //new_game.draw_lines.append(&mut crate::draw::draw_wireframe(&test_cube,GREEN));
        new_game.cur_lines_length = new_game.draw_lines.len();

        graphics.new_vertex_buffer_from_lines(&new_game.draw_lines);

        new_game
    }
    pub fn build_shapes(&self) -> Vec<Shape<V>> {
        vec![]
    }
    pub fn draw_stuff(&mut self) -> Vec<Option<draw::DrawLine<V::SubV>>> {
            //let mut shapes = self.shapes;
            //let camera = self.camera;
            //let clip_state = self.clip_state;
            //let extra_lines = self.extra_lines;
            //let face_scales = vec![0.1,0.3,0.5,0.7,1.0];
            //let face_scales = vec![0.3,0.5,0.8,1.0];
            let face_scales = vec![0.7];

            draw::update_shape_visibility(&self.camera, &mut self.shapes, &self.clip_state);
            self.clip_state.calc_in_front(&mut self.shapes,& self.camera.pos);
            draw::transform_draw_lines(
            {
                let mut lines = draw::calc_shapes_lines(&mut self.shapes,&face_scales,&self.clip_state);
                lines.append(&mut crate::draw::calc_lines_color_from_ref(
                    &self.shapes,
                    &self.extra_lines,CYAN));
                lines
            }, &self.camera)
    }
    pub fn game_update(&mut self, input : &mut Input) {
        if input.update {
            //if input.pressed.being_touched {
            if false {
                let shapes_len = self.shapes.len();
                self.shapes[shapes_len-1].rotate(0,-1,0.05);

            }
            input.update = true; //set to true for constant updating
        }

    }
    pub fn draw_update<'a,G>(&mut self, graphics : &mut G)
    where G : Graphics<'a,V::SubV>
    {

        // let start_instant = time::Instant::now();
        // let mut last_instant = time::Instant::now();
        // let mut game_duration : time::Duration;
        // let mut frame_duration : time::Duration;
        // input.closed = false;

        //while !input.closed && !input.swap_engine {
        // if input.closed || input.swap_engine {
        //     return;
        // }
        

            self.draw_lines = self.draw_stuff();

            //make new buffer if the number of lines changes
            if self.draw_lines.len() != self.cur_lines_length {
                graphics.new_vertex_buffer_from_lines(&self.draw_lines);
                //println!("New buffer! {} to {}",draw_lines.len(),cur_lines_length);
                self.cur_lines_length = self.draw_lines.len();
            }
            graphics.draw_lines(&self.draw_lines);
        
        //input.listen_events(event);

        //game_duration = time::Instant::now().duration_since(start_instant);
        //frame_duration = time::Instant::now().duration_since(last_instant);
        //last_instant = time::Instant::now();
        //input.update_camera(&mut self.camera, &frame_duration);
        //let shapes_len = shapes.len();
        //input.update_shape(&mut self.shapes[shapes_len-1]);
        
        //input.print_debug(&camera,&game_duration,&frame_duration,&mut clip_state,&shapes);
    }
}

impl Game<Vec3> {

}
pub fn build_shapes_3d() -> Vec<Shape<Vec3>> {

    build_level::build_lvl_1_3d()
    //build_level::build_test_scene_3d()
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
