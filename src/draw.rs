use serde::{Deserialize, Serialize};

use clipping::{clip_line_cube, clip_line_plane, ClipState};

use specs::{Join, ReadStorage};
pub use texture::{FaceTexture, ShapeTexture, Texture, TextureMapping};

use crate::components::*;
use crate::config::{DrawConfig, ViewConfig};
use crate::constants::{CARDINAL_COLORS, CURSOR_COLOR, SMALL_Z, Z0, ZERO, Z_NEAR};
use crate::ecs_utils::Componentable;
use crate::geometry::Face;
use crate::geometry::{Line, Shape};
use crate::graphics::colors::*;
use crate::vector::{weighted_sum, Field, Vec4, VectorTrait};

use self::clipping::boundaries::calc_boundaries;
use self::clipping::{
    clip_draw_lines, clip_line_cylinder, clip_line_sphere, clip_line_tube, make_boundaries,
};
use self::texture::shape_texture::draw_face_texture;
use self::visual_aids::calc_wireframe_lines;

pub mod clipping;
pub mod draw_line_collection;
pub mod systems;
pub mod texture;
pub mod visual_aids;

extern crate map_in_place;

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub enum ViewportShape {
    Cube,
    Sphere,
    Cylinder,
    Tube,
    None,
}

#[derive(Clone, Copy)]
pub struct DrawVertex<V>
where
    V: VectorTrait,
{
    pub vertex: V,
    pub color: Color,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DrawLine<V> {
    pub line: Line<V>,
    pub color: Color,
}
impl<V: VectorTrait> DrawLine<V> {
    #[allow(dead_code)]
    pub fn map_line<F, U>(self, f: F) -> DrawLine<U>
    where
        U: VectorTrait,
        F: Fn(Line<V>) -> Line<U>,
    {
        DrawLine {
            line: f(self.line),
            color: self.color,
        }
    }
    pub fn get_draw_verts(&self) -> [DrawVertex<V>; 2] {
        [
            DrawVertex {
                vertex: self.line.0,
                color: self.color,
            },
            DrawVertex {
                vertex: self.line.1,
                color: self.color,
            },
        ]
    }
}

fn project<V>(focal: Field, v: V) -> V::SubV
where
    V: VectorTrait,
{
    let z = if V::is_close(v, V::ones() * Z0) {
        Z0 + SMALL_Z
    } else {
        v[-1]
    };
    v.project() * focal / z
}
fn view_transform<V: VectorTrait>(transform: &Transform<V, V::M>, point: V) -> V {
    transform.frame * (point - transform.pos)
}
//can likely remove camera here by calculating the plane from the transform, unless you want the
//camera's plane to differ from its position/heading
pub fn transform_line<V: VectorTrait>(
    line: Line<V>,
    transform: &Transform<V, V::M>,
    camera: &Camera<V>,
    view_config: &ViewConfig,
) -> Option<Line<V::SubV>>
where
    V: VectorTrait,
{
    let r = view_config.radius;
    let h = view_config.height;
    clip_line_plane(line, &camera.plane, Z_NEAR)
        .map(|l| l.map(|v| project(view_config.focal, view_transform(transform, v))))
        .and_then(|l| match view_config.viewport_shape {
            ViewportShape::Cube => clip_line_cube(l, r),
            ViewportShape::Sphere => clip_line_sphere(l, r),
            ViewportShape::Cylinder => clip_line_cylinder(l, r, h),
            ViewportShape::Tube => clip_line_tube(l, r),
            ViewportShape::None => Some(l),
        })
}

#[derive(Default)]
pub struct DrawLineList<V>(pub Vec<DrawLine<V>>);
impl<V: VectorTrait> DrawLineList<V> {
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.0.len()
    }
    #[allow(dead_code)]
    pub fn map<F, U>(&self, f: F) -> DrawLineList<U>
    where
        U: VectorTrait,
        F: Fn(DrawLine<V>) -> DrawLine<U>,
    {
        DrawLineList(self.0.iter().map(|l| f(l.clone())).collect()) //another questionable clone
    }
    #[allow(dead_code)]
    pub fn flat_map<F, U>(&self, f: F) -> DrawLineList<U>
    where
        U: VectorTrait,
        F: Fn(DrawLine<V>) -> Option<DrawLine<U>>,
    {
        DrawLineList(self.0.iter().flat_map(|l| f(l.clone())).collect()) //another questionable clone
    }
}

//apply transform line to the lines in draw_lines
pub fn transform_draw_line<V: VectorTrait>(
    draw_line: DrawLine<V>,
    transform: &Transform<V, V::M>,
    camera: &Camera<V>,
    view_config: &ViewConfig,
) -> Option<DrawLine<V::SubV>> {
    transform_line(draw_line.line, transform, camera, view_config).map(|line| DrawLine {
        line,
        color: draw_line.color,
    })
}

pub fn draw_cursor<U: VectorTrait>(shape: &Shape<U>) -> impl Iterator<Item = DrawLine<U>> {
    calc_wireframe_lines(shape)
        .into_iter()
        .map(|line| DrawLine {
            line,
            color: CURSOR_COLOR,
        })
}

//updates clipping boundaries and face visibility based on normals
// mutated: shape_clip_state boundaries and face_visibility
pub fn update_shape_visibility<V: VectorTrait>(
    camera_pos: V,
    shape: &Shape<V>,
    shape_clip_state: &mut ShapeClipState<V>,
    clip_state: &ClipState<V>,
) {
    //update shape visibility and boundaries
    // build face visibility vec if empty
    if shape_clip_state.face_visibility.is_empty() {
        for face in shape.faces.iter() {
            shape_clip_state
                .face_visibility
                .push(get_face_visibility::<V>(
                    face,
                    camera_pos,
                    shape_clip_state.transparent | face.two_sided,
                ));
        }
    } else {
        for (face, visible) in shape
            .faces
            .iter()
            .zip(shape_clip_state.face_visibility.iter_mut())
        {
            *visible = get_face_visibility(
                face,
                camera_pos,
                shape_clip_state.transparent | face.two_sided,
            );
        }
    }

    //calculate boundaries for clipping
    if clip_state.clipping_enabled {
        shape_clip_state.boundaries =
            calc_boundaries(camera_pos, shape, &shape_clip_state.face_visibility);
    }
}

pub fn get_face_visibility<V: VectorTrait>(face: &Face<V>, camera_pos: V, two_sided: bool) -> bool {
    // TODO: this commented out condition is intended to eliminate faces that the camera is not facing, but can cause some artifacts
    // the threshold can probably be calculated from the focal length + viewport size; focal length = infinity corresponding to threshold of zero
    // (face.normal().dot(camera_dir) < 0.8)
    two_sided || (face.plane().point_signed_distance(camera_pos) > ZERO)
}

pub fn normal_to_color<V: VectorTrait>(normal: V) -> Color {
    weighted_sum(normal.into_iter().enumerate().map(|(i, n_i)| {
        let color = CARDINAL_COLORS[if n_i > ZERO { i } else { i + (V::DIM as usize) }];
        (Vec4::from(color), n_i.abs())
    }))
    .into()
}

pub type Scratch<T> = (Vec<T>, Vec<T>);

// this exists to make clippy happy in the below fn
type ShapeComponentStorage<'a, V, M> = (
    &'a ReadStorage<'a, Shape<V>>,
    &'a ReadStorage<'a, Transform<V, M>>,
    &'a ReadStorage<'a, ShapeTexture<V>>,
    &'a ReadStorage<'a, ShapeClipState<V>>,
);


type OldShapeComponentStorage<'a, V> = (
    &'a ReadStorage<'a, Shape<V>>,
    &'a ReadStorage<'a, ShapeTexture<V>>,
    &'a ReadStorage<'a, ShapeClipState<V>>,
);

pub fn calc_shapes_lines<V>(
    write_lines: &mut Vec<DrawLine<V>>,
    scratch: &mut Scratch<DrawLine<V>>,
    line_scratch: &mut Scratch<Line<V>>,
    shape_components: ShapeComponentStorage<V, V::M>,
    clip_state: &ClipState<V>,
    draw_config: &DrawConfig,
) where
    V: VectorTrait + Componentable,
    V::SubV: Componentable,
    V::M: Componentable,
{
    //compute lines for each shape
    for (shape, shape_transform, shape_texture, shape_clip_state) in shape_components.join() {
        scratch.0.clear();
        //get lines from each face
        for (face, &visible, face_texture) in izip!(
            shape.faces.iter(),
            shape_clip_state.face_visibility.iter(),
            shape_texture.face_textures.iter()
        ) {
            scratch.0.extend(draw_face_texture::<V>(
                face_texture,
                face,
                shape,
                shape_transform,
                &[draw_config.face_scale],
                visible,
                draw_config
                    .color_by_orientation
                    .then(|| normal_to_color(face.normal())),
            ))
        }

        if clip_state.clipping_enabled {
            let clip_states_in_front =
                shape_clip_state
                    .in_front
                    .iter()
                    .map(|&e| match shape_components.3.get(e) {
                        Some(s) => s,
                        None => panic!("Invalid entity {} found in shape_clip_state", e.id()),
                    });

            clip_draw_lines(
                &scratch.0,
                write_lines,
                line_scratch,
                make_boundaries(clip_states_in_front),
            );
        } else {
            write_lines.append(&mut scratch.0);
        }
    }
}
