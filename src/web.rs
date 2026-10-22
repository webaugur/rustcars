//! Browser entry point (feature `web`, target wasm32). The UI runs entirely in the visitor's
//! browser: it cannot reach the server's USB devices or GQRX TCP port. Device access must go
//! through the server-side bridge (`rustcars-web`, feature `web-server`) over WebSocket.
use wasm_bindgen::JsCast;

/// Mounts the demo on `<canvas id="rustcars_canvas">` (see index.html).
pub fn start() {
    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .and_then(|w| w.document())
            .expect("no document");
        let canvas = document
            .get_element_by_id("rustcars_canvas")
            .expect("missing #rustcars_canvas")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("#rustcars_canvas is not a canvas");
        let result = eframe::WebRunner::new()
            .start(
                canvas,
                eframe::WebOptions::default(),
                Box::new(|cc| Ok(Box::new(crate::DemoApp::new(&cc.egui_ctx)))),
            )
            .await;
        if let Err(e) = result {
            web_sys::console::error_1(&e);
        }
    });
}
