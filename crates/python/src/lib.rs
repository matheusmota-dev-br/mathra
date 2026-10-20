use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

fn convert(tex: &str, locale: &str, display: bool, strict: bool) -> PyResult<mathra::Output> {
    let locale = locale.parse().map_err(PyValueError::new_err)?;
    mathra::convert(tex, &mathra::Options { locale, display, strict }).map_err(|e| PyValueError::new_err(e.to_string()))
}

/// Converts math LaTeX to SVG.
#[pyfunction]
#[pyo3(signature = (tex, locale = "en", *, display = true, strict = false))]
fn tex_to_svg(tex: &str, locale: &str, display: bool, strict: bool) -> PyResult<String> {
    convert(tex, locale, display, strict).map(|r| r.svg)
}

/// Tree of meaning as JSON.
#[pyfunction]
#[pyo3(signature = (tex, locale = "en", *, display = true, strict = false))]
fn tex_to_tree(tex: &str, locale: &str, display: bool, strict: bool) -> PyResult<String> {
    convert(tex, locale, display, strict).map(|r| r.tree.to_json())
}

/// Every output as JSON; `mathra.render` turns it into a dict.
#[pyfunction]
#[pyo3(signature = (tex, locale = "en", *, display = true, strict = false))]
fn render(tex: &str, locale: &str, display: bool, strict: bool) -> PyResult<String> {
    convert(tex, locale, display, strict).map(|r| r.to_json())
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
    m.add_function(wrap_pyfunction!(tex_to_tree, m)?)?;
    m.add_function(wrap_pyfunction!(render, m)?)?;
    m.add_function(wrap_pyfunction!(locales, m)?)?;
    // the Python package has its own version (crates/python/Cargo.toml); the core's is `core_version`
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add("core_version", mathra::VERSION)?;
    Ok(())
}
