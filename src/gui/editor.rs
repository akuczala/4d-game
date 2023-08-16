use imgui::{Condition, StyleColor, Ui};
use specs::{World, WorldExt};

use crate::{
    components::{MaybeSelected, Player, Transform},
    ecs_utils::Componentable,
    graphics::colors::{RED, WHITE},
    vector::VectorTrait,
};

use super::{GuiInitArgs, GuiState, ListBoxIndex};

#[allow(dead_code)]
pub fn add_ref_shape_selector_broken(ui: &Ui, init_args: &GuiInitArgs, state: &mut GuiState) {
    let items = &init_args
        .shape_names
        .iter()
        .enumerate()
        // for some reason, I can't get this to consistently work without an empty id error
        // see https://github.com/ocornut/imgui/blob/master/docs/FAQ.md
        // but it shouldn't be this annoying
        // as an example, if we set the list to ["ok", "so"], we get the error???
        .map(|(i, s)| format!("{}##foo{}ffff", s, i))
        .collect::<Vec<_>>();
    let items = &items.iter().collect::<Vec<_>>();
    ui.list_box("Aggggh", &mut state.item, items, 10);
}

pub fn add_ref_shape_selector(ui: &Ui, init_args: &GuiInitArgs, state: &mut GuiState) {
    if ui.is_key_released(imgui::Key::RightArrow) {
        state.item = (state.item + 1).rem_euclid(init_args.shape_names.len() as ListBoxIndex);
    }
    if ui.is_key_released(imgui::Key::LeftArrow) {
        state.item = (state.item - 1).rem_euclid(init_args.shape_names.len() as ListBoxIndex);
    }
    let items = &init_args.shape_names;
    //let items = &items.iter().collect::<Vec<_>>();
    for (i, item) in items.iter().enumerate() {
        let color = if (i as ListBoxIndex) == state.item {
            ui.push_style_color(StyleColor::Text, *RED.get_arr())
        } else {
            ui.push_style_color(StyleColor::Text, *WHITE.get_arr())
        };
        ui.text(item);
        color.pop();
    }
}

pub fn make_info_string<V>(world: &World) -> String
where
    V: VectorTrait + Componentable,
    V::M: Componentable,
{
    let player = world.read_resource::<Player>();
    let transforms = world.read_storage();
    let maybe_selected_storage = world.read_storage();
    let maybe_selected: &MaybeSelected = maybe_selected_storage.get(player.0).unwrap();
    if let Some(ref selected) = maybe_selected.0 {
        let selected_transform: &Transform<V, V::M> = transforms.get(selected.entity).unwrap();
        format!(
            "pos: {}\nframe: \n{}\nscale: {:?}",
            selected_transform.pos, selected_transform.frame, selected_transform.scale
        )
    } else {
        "Nothing selected".to_string()
    }
}

pub fn editor_gui(
    _: &mut bool,
    ui: &Ui,
    state: &mut GuiState,
    init_args: &GuiInitArgs,
    info_string: &String,
) {
    ui.window("Something##oKkk")
        .position([0., 0.], Condition::Appearing)
        .size([190.0, 500.0], Condition::FirstUseEver)
        .always_auto_resize(true)
        .bg_alpha(0.75)
        .title_bar(false)
        .resizable(false)
        .scroll_bar(false)
        .menu_bar(false)
        .build(|| {
            add_ref_shape_selector(ui, init_args, state);
            ui.text(info_string)
        });
}
