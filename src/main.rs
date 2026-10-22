//! `cargo run` opens the LCARS demo window.
fn main() -> eframe::Result {
    let t = rustcars::LcarsTheme::default();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_title("rustcars LCARS demo"),
        ..Default::default()
    };
    let mut setup = true;
    eframe::run_ui_native("rustcars", options, move |ui, _frame| {
        if setup {
            rustcars::install_lcars_font(ui.ctx());
            rustcars::apply_theme(ui.ctx(), &t);
            setup = false;
        }
        rustcars::demo(ui, &t);
    })
}
