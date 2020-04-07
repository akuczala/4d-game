pub mod game3d;
pub mod game4d; 

//use crate::vector::{VectorTrait,MatrixTrait};
use crate::graphics::Graphics;
//use draw;
use crate::geometry::{Shape,Line};
use crate::clipping;
use crate::vector::{VectorTrait};
use crate::input::Input;
use crate::colors::*;
use crate::draw::Camera;
use crate::draw;


pub trait Game<V : VectorTrait> {
    fn new() -> Self;
    fn get_clip_state(&mut self) -> &mut clipping::ClipState<V>;
    fn get_camera(&mut self) -> &mut Camera<V>;
    fn get_shapes(&mut self) -> &mut Vec<Shape<V>>;
    fn get_extra_lines(&mut self) -> &mut Vec<Line<V>>;
    fn draw_stuff(&mut self) -> Vec<Option<draw::DrawLine<V::SubV>>> {
            let shapes = self.get_shapes();
            let camera = self.get_camera();
            let clip_state = self.get_clip_state();
            let extra_lines = self.get_extra_lines();
            //let face_scales = vec![0.1,0.3,0.5,0.7,1.0];
            //let face_scales = vec![0.3,0.5,0.8,1.0];
            let face_scales = vec![0.7];

            draw::update_shape_visibility(camera, shapes, clip_state);
            clip_state.calc_in_front(&shapes,&camera.pos);
            draw::transform_draw_lines(
            {
                let mut lines = draw::calc_shapes_lines(shapes,&face_scales,&clip_state);
                lines.append(&mut crate::draw::calc_lines_color_from_ref(
                    &shapes,
                    &extra_lines,CYAN));
                lines
            }, &camera)
        }
    // fn game<'a,G>(&mut self, mut graphics : G, input : &mut Input)
    // where G : Graphics<'a,V::SubV>
    // {


    //     let mut clip_state = clipping::ClipState::new(&self.shapes);
    //     // let test_cube = buildshapes::build_cube_3d(1.0)
    //     //     .set_pos(&Vec3::new(0.0,0.0,0.0));

        
    //     let mut draw_lines = self.draw_stuff();
    //     //draw_lines.append(&mut crate::draw::draw_wireframe(&test_cube,GREEN));
    //     let mut cur_lines_length = draw_lines.len();
        
    //     graphics.new_vertex_buffer_from_lines(&draw_lines);

    //     let start_instant = time::Instant::now();
    //     let mut last_instant = time::Instant::now();
    //     let mut game_duration : time::Duration;
    //     let mut frame_duration : time::Duration;
    //     input.closed = false;

    //     events_loop.run(move |event, _, control_flow| {
    //     //while !input.closed && !input.swap_engine {
    //         if input.closed || input.swap_engine {
    //             return;
    //         }
    //         if input.update {
    //             //if input.pressed.being_touched {
    //             if false {
    //                 let shapes_len = shapes.len();
    //                 shapes[shapes_len-1].rotate(0,-1,0.05);

    //             }

    //             draw_lines = draw_stuff(&camera, &mut shapes, &mut clip_state, &extra_lines);

    //             //make new buffer if the number of lines changes
    //             if draw_lines.len() != cur_lines_length {
    //                 graphics.new_vertex_buffer_from_lines(&draw_lines);
    //                 //println!("New buffer! {} to {}",draw_lines.len(),cur_lines_length);
    //                 cur_lines_length = draw_lines.len();
    //             }
    //             graphics.draw_lines(&draw_lines);

                
    //             input.update = true; //set to true for constant updating
    //         }
            
    //         input.listen_events(event);

    //         game_duration = time::Instant::now().duration_since(start_instant);
    //         frame_duration = time::Instant::now().duration_since(last_instant);
    //         last_instant = time::Instant::now();
    //         input.update_camera(&mut camera, &frame_duration);
    //         let shapes_len = shapes.len();
    //         input.update_shape(&mut shapes[shapes_len-1]);
            
    //         input.print_debug(&camera,&game_duration,&frame_duration,&mut clip_state,&shapes);

            
    //     });
    //}
}