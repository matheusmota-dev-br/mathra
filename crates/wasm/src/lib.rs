use wasm_bindgen::prelude::*;

/// What JavaScript gets, without JavaScript types: testable outside the browser.
pub fn convert(tex: &str, locale: Option<&str>) -> Result<mathra::Output, String> {
    convert_with(tex, locale, true, false)
}

pub fn convert_with(tex: &str, locale: Option<&str>, display: bool, strict: bool) -> Result<mathra::Output, String> {
    let locale = locale.map_or(Ok(mathra::Locale::default()), str::parse)?;
    mathra::convert(tex, &mathra::Options { locale, display, strict }).map_err(|e| e.to_string())
}

/// Everything at once, as JSON: `svg`, `speech`, `text`, `tree`, `applied` and `warnings`.
pub fn render_json(tex: &str, locale: Option<&str>, display: bool, strict: bool) -> Result<String, String> {
    convert_with(tex, locale, display, strict).map(|r| r.to_json())
}

/// Converts math LaTeX to SVG. `locale`: a language code such as "en" (default), "pt-BR", "es" or "ja";
/// `display` defaults to true, `strict` to false.
#[wasm_bindgen(js_name = texToSvg)]
pub fn tex_to_svg(tex: &str, locale: Option<String>, display: Option<bool>, strict: Option<bool>) -> Result<String, JsError> {
    convert_with(tex, locale.as_deref(), display.unwrap_or(true), strict.unwrap_or(false)).map(|r| r.svg).map_err(|e| JsError::new(&e))
}

/// Tree of meaning as JSON.
#[wasm_bindgen(js_name = texToTree)]
pub fn tex_to_tree(tex: &str, locale: Option<String>) -> Result<String, JsError> {
    convert(tex, locale.as_deref()).map(|r| r.tree.to_json()).map_err(|e| JsError::new(&e))
}

/// The supported languages as JSON: `[{"code":"en","name":"English"}, …]`.
pub fn locales_json() -> String {
    let list: Vec<serde_json::Value> = mathra::Locale::ALL.iter().map(|l| serde_json::json!({ "code": l.code(), "name": l.name() })).collect();
    serde_json::Value::Array(list).to_string()
}

#[wasm_bindgen]
pub fn locales() -> String {
    locales_json()
}

/// Every output in one JSON string (see `render_json`). `display` defaults to true, `strict` to false.
#[wasm_bindgen]
pub fn render(tex: &str, locale: Option<String>, display: Option<bool>, strict: Option<bool>) -> Result<String, JsError> {
    render_json(tex, locale.as_deref(), display.unwrap_or(true), strict.unwrap_or(false)).map_err(|e| JsError::new(&e))
}
