use glium::Display;
use crate::geometry::{Shape,Line};
use crate::geometry::buildshapes;
use crate::vector::{VectorTrait,Vec3,Vec4};
use crate::colors::*;
use crate::build_level;
use crate::camera::Camera;
use crate::clipping::ClipState;
use crate::input::Input;
use crate::graphics::Graphics;
use crate::draw;

use crate::fps::FPSFloat;

use specs::prelude::*;

pub struct Game<V : VectorTrait> {
    pub world : World,
    //pub shapes : Vec<Shape<V>>,
    pub extra_lines : Vec<Line<V>>,
    //pub camera : Camera<V>,
    //pub clip_state : ClipState<V>,
    //pub draw_lines : Vec<Option<draw::DrawLine<V::SubV>>>,
    pub cur_lines_length : usize
}

impl<V> Game<V>
where V: VectorTrait
{
    pub fn new(shapes : Vec<Shape<V>>) -> Self
    {
        let mut world = World::new();
        world.register::<Shape<V>>();

        //change to into_iter, remove cloning
        let shapes_len = shapes.len();
        for shape in shapes.into_iter() {
            world.create_entity().with(shape).build();
        }

        let extra_lines : Vec<Line<V>> = Vec::new();
        let mut camera = Camera::new(V::zero()-V::one_hot(-1)*0.);
        use crate::vector::Rotatable;
        camera.rotate(-2,-1,3.14159/2.);
        
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
        Game {
            world,
            extra_lines,
            //camera,
            //clip_state : ClipState::new(0),
            //draw_lines : vec![],
            cur_lines_length,
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
    pub fn draw_stuff(&mut self) -> Vec<Option<draw::DrawLine<V::SubV>>> {
            //let mut shapes = self.shapes;
            //let camera = self.camera;
            //let clip_state = self.clip_state;
            //let extra_lines = self.extra_lines;
            //let face_scales = vec![0.1,0.3,0.5,0.7,1.0];
            //let face_scales = vec![0.3,0.5,0.8,1.0];
            //let face_scales = vec![0.8,0.9];

            //draw::update_shape_visibility(&self.camera, &mut self.shapes, &self.clip_state);
            //self.clip_state.calc_in_front(self.get_shapes(),& self.camera.pos);

            //draw lines
            //let face_scales = vec![0.2,0.5,0.7,0.9];
            

            //make this a system or two or three
            // draw::transform_draw_lines(
            // {
            //     let mut lines = draw::calc_shapes_lines(self.get_shapes(),&face_scales,&self.clip_state);
            //     lines.append(&mut crate::draw::calc_lines_color_from_ref(
            //         self.get_shapes(),
            //         &self.extra_lines,CYAN));
            //     lines
            // }, &self.world.system_data())
            unimplemented!()
    }
    pub fn game_update(&mut self, input : &mut Input, frame_len : FPSFloat ) {
        

        if input.update {
            //if input.pressed.being_touched {
            if true {
                let shapes_len = self.get_shapes().as_slice().len();
                self.get_mut_shapes().as_mut_slice()[shapes_len-1].rotate(0,-1,0.05);

            }
            //DISABLE CAMERA INPUT
            //input.update_camera(&mut self.camera, frame_len);

        let shapes_len = self.get_shapes().as_slice().len();
        input.update_shape(&mut self.get_mut_shapes().as_mut_slice()[shapes_len-1]);
        
        //input.print_debug(&self.camera,&game_duration,&frame_duration,&mut clip_state,&shapes);
            input.update = true; //set to true for constant updating
        }

    }
    pub fn draw_update<G>(&mut self, graphics : &mut G, display : &Display)
    where G : Graphics<V::SubV>
    {
        //self.draw_lines = self.draw_stuff();

        //V::zero() tells the system what dimension it's working in.
        //would like to do this with phantom data or something
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
            graphics.new_vertex_buffer_from_lines(draw_lines,display);
            //println!("New buffer! {} to {}",draw_lines.len(),cur_lines_length);
            self.cur_lines_length = draw_lines.len();
        }
        graphics.draw_lines(&draw_lines,display);
    }
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
