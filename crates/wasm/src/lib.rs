use wasm_bindgen::prelude::*;

/// What JavaScript gets, without JavaScript types: testable outside the browser.
pub fn convert(tex: &str, locale: Option<&str>) -> Result<mathra::Output, String> {
    let locale = locale.map_or(Ok(mathra::Locale::default()), str::parse)?;
    mathra::convert(tex, &mathra::Options { locale, ..Default::default() }).map_err(|e| e.to_string())
}

/// Converts math LaTeX to SVG. `locale`: "en" (default) or "pt-BR".
#[wasm_bindgen(js_name = texToSvg)]
pub fn tex_to_svg(tex: &str, locale: Option<String>) -> Result<String, JsError> {
    convert(tex, locale.as_deref()).map(|r| r.svg).map_err(|e| JsError::new(&e))
}

/// Tree of meaning as JSON.
#[wasm_bindgen(js_name = texToTree)]
pub fn tex_to_tree(tex: &str, locale: Option<String>) -> Result<String, JsError> {
    convert(tex, locale.as_deref()).map(|r| r.tree.to_json()).map_err(|e| JsError::new(&e))
}
