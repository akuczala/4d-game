use crate::{
    components::*,
    ecs_utils::Componentable,
    input::{ShapeManipulationMode, ShapeManipulationState},
    vector::VectorTrait,
};
use specs::prelude::*;

pub fn make_debug_string<V>(world: &World) -> String
where
    V: VectorTrait + Componentable,
    V::M: Componentable,
{
    let mut debug_text = "".to_string();

    let player = world.read_resource::<Player>();
    // the let statements for storage here are needed to avoid temporary borrowing
    let maybe_target_storage = world.read_storage::<MaybeTarget<V>>();
    let maybe_target = maybe_target_storage
        .get(player.0)
        .expect("player has no target");
    let maybe_selected_storage = world.read_storage::<MaybeSelected>();
    let maybe_selected = maybe_selected_storage
        .get(player.0)
        .expect("player has no selection component");

    let shapes = world.read_component::<Shape<V>>();
    let transforms = world.read_component::<Transform<V, V::M>>();
    let mut normals: Vec<V> = vec![];
    for shape in (&shapes).join() {
        for face in shape.faces.iter() {
            normals.push(face.normal())
        }
    }
    //let input = world.read_resource::<Input>();

    let debug_strings: Vec<String> = vec![
        // format!(
        //     "Integrated mouse: {:?}\n",
        //     input.mouse.integrated_mouse_dpos
        // ),
        // format!(
        //     "Integrated scroll: {:?}\n",
        //     input.mouse.integrated_scroll_dpos
        // ),
        match maybe_target {
            MaybeTarget(Some(target)) => format!(
                "target: {}, {}, {}\n",
                target.entity.id(),
                target.distance,
                target.point
            ),
            MaybeTarget(None) => "No target\n".to_string(),
        },
        match maybe_selected {
            MaybeSelected(Some(selected)) => {
                //let bbox_storage = world.read_storage::<BBox<V>>();
                //let selected_bbox = bbox_storage.get(selected.entity).expect("selected entity has no bbox");
                let selected_transform = transforms.get(selected.entity).expect("Nope");
                //let (frame, scaling) = selected_transform.decompose_rotation_scaling();
                let (frame, scaling) = (selected_transform.frame, selected_transform.scale);
                //let bbox_info = format!("target ({}) bbox: {:?}\n",selected.entity.id(), *selected_bbox);
                let frame_info = format!(
                    "target frame: {}\n, {}\n{:?}\n",
                    selected.entity.id(),
                    frame,
                    scaling
                );

                let manip_state = world.read_resource::<ShapeManipulationState<V, V::M>>();
                let manip_info = match manip_state.mode {
                    ShapeManipulationMode::Translate(v) => format!("Translate: {}", v),
                    ShapeManipulationMode::Rotate(a) => format!("Rotate: {:.2}", a),
                    ShapeManipulationMode::Scale(s) => format!("Scale: {:?}", s),
                    ShapeManipulationMode::Free(_t) => "Free".to_string(),
                };
                let axes_info = manip_state
                    .locked_axes
                    .iter()
                    .fold("Axes:".to_string(), |s, &i| s + &i.to_string());
                let _clip_info = {
                    let scs = world.read_component::<ShapeClipState<V>>();
                    let shape_clip_state = scs.get(selected.entity).unwrap();
                    format!(
                        "In front: {:?}\nSeparators: {:?}",
                        shape_clip_state.in_front, shape_clip_state.separators
                    )
                };
                let dist_info: String = {
                    //let shapes = world.read_component::<Shape<V>>();
                    //let shape = shapes.get(selected.entity).unwrap();
                    let player_pos = transforms.get(player.0).unwrap().pos;
                    //format!("Distance: {}", shape.point_signed_distance(player_pos))
                    (&shapes)
                        .join()
                        .map(|shape| {
                            format!("Distance: {}", shape.point_signed_distance(player_pos))
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                };
                format!(
                    "{}\n{}\n{}\n{}\n",
                    axes_info, frame_info, manip_info, dist_info
                )
            }
            MaybeSelected(None) => "No selection\n".to_string(),
        },
        //crate::clipping::ShapeClipState::<V>::in_front_debug(world),
    ]
    .into_iter()
    //.chain(all_verts.into_iter().map(|v| format!{"::{}\n", v}))
    //.chain(normals.into_iter().map(|n| format!("{}\n", n)))
    .collect();

    //print draw lines
    //let draw_lines = world.read_resource::<DrawLineList<V::SubV>>();
    // for line in draw_lines.0.iter() {
    //     if let Some(ref l) = line {
    //         debug_strings.push(format!("{:}\n",l.line));
    //     }
    // }

    //concatenate all strings
    for string in debug_strings.into_iter() {
        debug_text = textwrap::fill(&format!("{}{}", debug_text, string), 40);
    }
    debug_text
}
