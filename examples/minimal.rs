//! Minimal rustcars app used by the manual (docs/manual/02-getting-started.md).
//! Run: `cargo run --example minimal`
use rustcars::{
    Cap, LcarsFrame, LcarsTheme, Mode, Side, apply_theme, install_lcars_font, lcars_button,
    mode_button, mode_list, readout,
};

struct App {
    theme: LcarsTheme,
    scan: Mode,
    rows: Vec<(String, Mode)>,
}

impl App {
    fn new(ctx: &egui::Context) -> Self {
        let theme = LcarsTheme::default();
        install_lcars_font(ctx); // Antonio, SIL OFL 1.1: ship assets/fonts/OFL.txt with your app
        apply_theme(ctx, &theme);
        Self {
            theme,
            scan: Mode::Inactive,
            rows: vec![
                ("1.031 Dalton".into(), Mode::Inactive),
                ("1.032 Hortonville".into(), Mode::Inactive),
            ],
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _f: &mut eframe::Frame) {
        let t = self.theme;
        let bg = egui::Frame::new().fill(t.bg).inner_margin(t.frame_gap);
        egui::CentralPanel::default().frame(bg).show(ui, |ui| {
            let mut frame = LcarsFrame::new(&t, Side::Left, 1.0);
            frame.min_height = ui.available_height();
            frame.show(ui, &t, |ui| self.content(ui, &t));
        });
    }
}

impl App {
    fn content(&mut self, ui: &mut egui::Ui, t: &LcarsTheme) {
        ui.horizontal(|ui| {
            let seg = egui::vec2(110.0, t.bar_thin);
            lcars_button(ui, t, "Home", t.primary, seg, Cap::Left, true);
            let size = egui::vec2(96.0, 56.0);
            mode_button(ui, t, "Scan", &mut self.scan, size);
        });
        mode_list(ui, t, "channels", &mut self.rows, 28.0);
        let busy = self.scan != Mode::Inactive;
        readout(ui, t, "851.388 MHZ", busy);
    }
}

fn main() -> eframe::Result {
    eframe::run_native(
        "rustcars minimal",
        eframe::NativeOptions::default(),
        Box::new(|cc| Ok(Box::new(App::new(&cc.egui_ctx)))),
    )
}
