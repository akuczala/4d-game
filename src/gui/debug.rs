use imgui::{Condition, Ui};

use crate::{fps::FPSFloat, gui::editor::add_ref_shape_selector};

use super::{GuiArgs, GuiInitArgs, GuiState};

pub fn frame_rate_text(ui: &Ui, frame_duration: FPSFloat) {
    ui.text(format!("FPS: {:0.0}", 1. / frame_duration));
}

pub fn debug_gui(
    _: &mut bool,
    ui: &Ui,
    ui_args: &mut GuiArgs,
    state: &mut GuiState,
    init_args: &GuiInitArgs,
) {
    ui.window("UnassumingName##butt")
        .position([0., 0.], Condition::Appearing)
        .size([190.0, 500.0], Condition::FirstUseEver)
        .always_auto_resize(true)
        .bg_alpha(0.75)
        .title_bar(false)
        .resizable(false)
        .scroll_bar(false)
        .menu_bar(false)
        .build(|| {
            if let GuiArgs::Debug {
                ref frame_duration,
                ref debug_text,
            } = ui_args
            {
                frame_rate_text(ui, *frame_duration);
                ui.text(debug_text);
            };
            if ui.radio_button_bool("I toggle my state on click", state.checked) {
                state.checked = !state.checked; // flip state on click
                state.text = "*** Toggling radio button was clicked".to_string();
            }
            add_ref_shape_selector(ui, init_args, state);
        });
}
