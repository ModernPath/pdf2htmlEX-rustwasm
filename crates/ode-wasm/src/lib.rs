use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn convert_pdf(data: &[u8]) -> Result<String, JsValue> {
    Ok("WASM stub".to_string())
}
