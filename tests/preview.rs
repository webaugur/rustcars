//! Renders the demo with egui_kittest + wgpu (software Vulkan/lavapipe works headless).
//! `cargo test --test preview -- --nocapture` writes docs/preview.png
use egui_kittest::Harness;

#[test]
fn render_preview() {
    let t = rustcars::LcarsTheme::default();
    let mut setup = true;
    let mut harness = Harness::builder()
        .with_size(egui::vec2(1280.0, 800.0))
        .wgpu()
        .build_ui(move |ui| {
            if setup {
                rustcars::install_lcars_font(ui.ctx());
                rustcars::apply_theme(ui.ctx(), &t);
                setup = false;
            }
            rustcars::demo(ui, &t);
        });
    harness.run_steps(4);
    let mut img = harness.render().expect("wgpu render failed");
    // build_ui leaves a transparent margin around the root Ui; flatten onto LCARS black.
    for px in img.pixels_mut() {
        let a = px[3] as u32;
        for k in 0..3 {
            px[k] = (px[k] as u32 * a / 255) as u8;
        }
        px[3] = 255;
    }
    img.save(concat!(env!("CARGO_MANIFEST_DIR"), "/docs/preview.png"))
        .unwrap();
    // Regression snapshot (tests/snapshots/lcars_demo.png); UPDATE_SNAPSHOTS=1 to refresh.
    harness.snapshot("lcars_demo");
}
