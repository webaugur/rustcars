//! `cargo run` opens the LCARS demo window (desktop).
//! With `--features web` on wasm32 (via Trunk) the same UI starts in the browser instead.

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_title("rustcars LCARS demo"),
        ..Default::default()
    };
    eframe::run_native(
        "rustcars",
        options,
        Box::new(|cc| Ok(Box::new(rustcars::DemoApp::new(&cc.egui_ctx)))),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    #[cfg(feature = "web")]
    rustcars::web::start();
}
