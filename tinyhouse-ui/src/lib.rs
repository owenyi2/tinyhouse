mod app;
pub use app::TinyhouseApp;

use eframe::wasm_bindgen::{self, prelude::*};
use web_sys;

#[wasm_bindgen]
pub async fn start(canvas_id: web_sys::HtmlCanvasElement) -> Result<(), wasm_bindgen::JsValue> {
    eframe::WebRunner::new()
        .start(
            canvas_id,
            eframe::WebOptions::default(),
            Box::new(|cc| Ok(Box::new(TinyhouseApp::new(cc)))),
        )
        .await
}
