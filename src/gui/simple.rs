use imgui::{ProgressBar, Ui};

use super::GuiArgs;

pub fn simple_gui(_: &mut bool, ui: &Ui, ui_args: &mut GuiArgs) {
    use imgui::Condition;
    ui.window("Press M to toggle mouse control")
        .position([0., 0.], Condition::Appearing)
        .size([190.0, 110.0], Condition::FirstUseEver)
        .bg_alpha(0.75)
        .title_bar(false)
        .resizable(false)
        .scroll_bar(false)
        .menu_bar(false)
        .build(|| {
            if let GuiArgs::Simple {
                ref frame_duration,
                ref coins_collected,
                ref coins_left,
            } = ui_args
            {
                let total_coins = coins_left + coins_collected;
                let coin_text = format!("Coins: {}/{}", coins_collected, total_coins);
                ui.text(format!("FPS: {:0.0}", 1. / frame_duration));
                ui.text(coin_text);
                ProgressBar::new((*coins_collected as f32) / (total_coins as f32))
                    //.size([200.0, 20.0])
                    .build(ui);
                ui.text("Press M to toggle mouse");
                ui.text("Backspace toggles 3D/4D");
            };
        });
}
