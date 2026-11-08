use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

#[allow(clippy::too_many_arguments)]
fn convert(
    tex: &str,
    locale: &str,
    display: bool,
    strict: bool,
    input: &str,
    braille: Option<&str>,
    animate: bool,
    steps: bool,
    font: Option<usize>,
    max_width: Option<f32>,
) -> PyResult<mathra::Output> {
    let locale = locale.parse().map_err(PyValueError::new_err)?;
    let input = input.parse().map_err(PyValueError::new_err)?;
    let braille = braille.map(str::parse).transpose().map_err(PyValueError::new_err)?;
    let font = match font {
        None => mathra::font::MathFont::default(),
        Some(id) => *mathra::font::MathFont::loaded().get(id).ok_or_else(|| PyValueError::new_err(format!("no font loaded with id {id}")))?,
    };
    mathra::convert(tex, &mathra::Options { locale, display, strict, input, braille, animate, steps, font, max_width, ..Default::default() })
        .map_err(|e| PyValueError::new_err(e.to_string()))
}

/// Loads an OpenType math font from its bytes: its id, to pass as `font`, and its name.
#[pyfunction]
fn load_font(data: Vec<u8>) -> PyResult<(usize, String)> {
    let font = mathra::font::MathFont::load(data).map_err(|e| PyValueError::new_err(e.to_string()))?;
    let id = mathra::font::MathFont::loaded().iter().position(|f| *f == font).expect("a loaded font is listed");
    Ok((id, font.name().to_string()))
}

/// Converts an equation (LaTeX, MathML or AsciiMath, by `input`) to SVG.
#[pyfunction]
#[pyo3(signature = (tex, locale = "en", *, display = true, strict = false, input = "latex", braille = None, animate = false, steps = false, font = None, max_width = None))]
#[allow(clippy::too_many_arguments)]
fn tex_to_svg(
    tex: &str,
    locale: &str,
    display: bool,
    strict: bool,
    input: &str,
    braille: Option<&str>,
    animate: bool,
    steps: bool,
    font: Option<usize>,
    max_width: Option<f32>,
) -> PyResult<String> {
    convert(tex, locale, display, strict, input, braille, animate, steps, font, max_width).map(|r| r.svg)
}

/// The equation as PNG bytes: `dpi` over a 12pt body (an em is dpi × 12 / 72 pixels), `background`
/// and `color` as CSS colors (transparent and `#111` by default).
#[pyfunction]
#[pyo3(signature = (tex, locale = "en", *, display = true, strict = false, input = "latex", braille = None, font = None, dpi = 300.0, background = None, color = "#111"))]
#[allow(clippy::too_many_arguments)]
fn tex_to_png(
    py: Python<'_>,
    tex: &str,
    locale: &str,
    display: bool,
    strict: bool,
    input: &str,
    braille: Option<&str>,
    font: Option<usize>,
    dpi: f32,
    background: Option<String>,
    color: &str,
) -> PyResult<Py<pyo3::types::PyBytes>> {
    let options = mathra::PngOptions { dpi, background, color: color.to_string() };
    options.check().map_err(|e| PyValueError::new_err(e.to_string()))?;
    let png = convert(tex, locale, display, strict, input, braille, false, false, font)?.png(&options).map_err(|e| PyValueError::new_err(e.to_string()))?;
    Ok(pyo3::types::PyBytes::new(py, &png).unbind())
}

/// Tree of meaning as JSON.
#[pyfunction]
#[pyo3(signature = (tex, locale = "en", *, display = true, strict = false, input = "latex", braille = None, animate = false, steps = false, font = None, max_width = None))]
#[allow(clippy::too_many_arguments)]
fn tex_to_tree(
    tex: &str,
    locale: &str,
    display: bool,
    strict: bool,
    input: &str,
    braille: Option<&str>,
    animate: bool,
    steps: bool,
    font: Option<usize>,
    max_width: Option<f32>,
) -> PyResult<String> {
    convert(tex, locale, display, strict, input, braille, animate, steps, font, max_width).map(|r| r.tree.to_json())
}

/// Every output as JSON; `mathra.render` turns it into a dict.
#[pyfunction]
#[pyo3(signature = (tex, locale = "en", *, display = true, strict = false, input = "latex", braille = None, animate = false, steps = false, font = None, max_width = None))]
#[allow(clippy::too_many_arguments)]
fn render(
    tex: &str,
    locale: &str,
    display: bool,
    strict: bool,
    input: &str,
    braille: Option<&str>,
    animate: bool,
    steps: bool,
    font: Option<usize>,
    max_width: Option<f32>,
) -> PyResult<String> {
    convert(tex, locale, display, strict, input, braille, animate, steps, font, max_width).map(|r| r.to_json())
}

/// The supported languages as (code, name) pairs, English first.
#[pyfunction]
fn locales() -> Vec<(&'static str, &'static str)> {
    mathra::Locale::ALL.iter().map(|l| (l.code(), l.name())).collect()
}

#[pymodule]
#[pyo3(name = "_mathra")]
fn mathra_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(tex_to_svg, m)?)?;
    m.add_function(wrap_pyfunction!(tex_to_png, m)?)?;
    m.add_function(wrap_pyfunction!(tex_to_tree, m)?)?;
    m.add_function(wrap_pyfunction!(render, m)?)?;
    m.add_function(wrap_pyfunction!(locales, m)?)?;
    m.add_function(wrap_pyfunction!(load_font, m)?)?;
    // the Python package has its own version (crates/python/Cargo.toml); the core's is `core_version`
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add("core_version", mathra::VERSION)?;
    Ok(())
}
