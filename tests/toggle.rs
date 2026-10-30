//! [PROJECT] P1b toggle buttons: thin border, ●/○ marker, contrast, headless click.
use egui::{Color32, vec2};
use rustcars::*;
use std::sync::mpsc;
use std::time::Duration;

const HUES: [Color32; 8] = [
    Color32::from_rgb(0x9C, 0x80, 0x5A),
    Color32::from_rgb(0x51, 0x6D, 0x8F),
    Color32::from_rgb(0x56, 0x60, 0x3A),
    Color32::from_rgb(0x70, 0x46, 0x3E),
    Color32::from_rgb(0x34, 0x5A, 0x54),
    Color32::from_rgb(0x60, 0x46, 0x66),
    Color32::from_rgb(0xA0, 0x5A, 0x50),
    Color32::from_rgb(0xD9, 0x42, 0x3A),
];

#[test]
fn marker_text() {
    assert_eq!(toggle_label("live", true), "● LIVE");
    assert_eq!(toggle_label("Data", false), "○ DATA");
}

#[test]
fn border_is_thin() {
    const { assert!(TOGGLE_BORDER > 1.0 && TOGGLE_BORDER <= 2.0) };
}

#[test]
fn toggle_contrast() {
    for h in HUES {
        for on in [true, false] {
            let (fill, border, ink) = toggle_colors(h, on);
            assert!(contrast(border, fill) >= 3.0, "{h:?} on={on} border");
            assert!(contrast(ink, fill) >= 4.5, "{h:?} on={on} label");
        }
    }
}

#[test]
fn headless_click_flips() {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let ctx = egui::Context::default();
        let t = LcarsTheme::default();
        let mut on = false;
        let run = |ctx: &egui::Context, on: &mut bool, input: egui::RawInput| {
            let mut changed = false;
            let mut out = ctx.run_ui(input, |ui| {
                ui.set_min_size(vec2(200.0, 60.0));
                changed |= toggle_button(ui, &t, "Live", on, HUES[6], vec2(80.0, 30.0)).changed();
            });
            out.textures_delta.clear();
            changed
        };
        run(&ctx, &mut on, egui::RawInput::default());
        let p = egui::pos2(40.0, 15.0);
        let ev = |pressed| egui::Event::PointerButton {
            pos: p,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: Default::default(),
        };
        let mk = |e: Vec<egui::Event>| egui::RawInput {
            events: e,
            ..Default::default()
        };
        run(
            &ctx,
            &mut on,
            mk(vec![egui::Event::PointerMoved(p), ev(true)]),
        );
        let changed = run(&ctx, &mut on, mk(vec![ev(false)]));
        tx.send((on, changed)).unwrap();
    });
    let (on, changed) = rx
        .recv_timeout(Duration::from_secs(10))
        .expect("frame run timed out");
    assert!(on && changed);
}

/// P1c: bar variants are square; the panel variants keep radius 4.
#[test]
fn bar_buttons_are_square() {
    let ctx = egui::Context::default();
    rustcars::install_lcars_font(&ctx);
    let t = rustcars::LcarsTheme::default();
    let mut out = egui::FullOutput::default();
    for _ in 0..2 {
        out = ctx.run_ui(egui::RawInput::default(), |ui| {
            let mut m = rustcars::Mode::Inactive;
            rustcars::bar_mode_button(ui, &t, "Copy", &mut m, egui::vec2(100.0, 40.0));
            let mut on = true;
            rustcars::bar_toggle_button(ui, &t, "Live", &mut on, rustcars::palette::BRICK, egui::vec2(100.0, 40.0));
        });
        out.textures_delta.clear();
    }
    let rects: Vec<_> = out.shapes.iter().filter_map(|c| match &c.shape { egui::Shape::Rect(r) => Some(r.clone()), _ => None }).collect();
    assert!(rects.len() >= 2);
    for r in rects {
        assert_eq!(r.corner_radius, egui::CornerRadius::ZERO, "{r:?}");
    }
}
