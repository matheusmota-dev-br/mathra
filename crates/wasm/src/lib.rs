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
    let r = convert_with(tex, locale, display, strict)?;
    let rules = |list: &[mathra::Applied]| -> Vec<serde_json::Value> { list.iter().map(|a| serde_json::json!({ "rule": a.rule, "description": a.description })).collect() };
    let tree: serde_json::Value = serde_json::from_str(&r.tree.to_json()).map_err(|e| e.to_string())?;
    let out = serde_json::json!({
        "svg": r.svg,
        "speech": r.speech,
        "text": r.text,
        "tree": tree,
        "applied": rules(&r.applied),
        "warnings": rules(&r.warnings),
    });
    Ok(out.to_string())
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

/// Every output in one JSON string (see `render_json`). `display` defaults to true, `strict` to false.
#[wasm_bindgen]
pub fn render(tex: &str, locale: Option<String>, display: Option<bool>, strict: Option<bool>) -> Result<String, JsError> {
    render_json(tex, locale.as_deref(), display.unwrap_or(true), strict.unwrap_or(false)).map_err(|e| JsError::new(&e))
}
