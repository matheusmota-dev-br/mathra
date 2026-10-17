use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

fn options(locale: &str) -> PyResult<mathra::Options> {
    let locale = locale.parse().map_err(PyValueError::new_err)?;
    Ok(mathra::Options { locale, ..Default::default() })
}

/// Converts math LaTeX to SVG.
#[pyfunction]
#[pyo3(signature = (tex, locale = "en"))]
fn tex_to_svg(tex: &str, locale: &str) -> PyResult<String> {
    mathra::convert(tex, &options(locale)?).map(|r| r.svg).map_err(|e| PyValueError::new_err(e.to_string()))
}

/// Tree of meaning as JSON.
#[pyfunction]
#[pyo3(signature = (tex, locale = "en"))]
fn tex_to_tree(tex: &str, locale: &str) -> PyResult<String> {
    mathra::convert(tex, &options(locale)?).map(|r| r.tree.to_json()).map_err(|e| PyValueError::new_err(e.to_string()))
}

#[pymodule]
#[pyo3(name = "_mathra")]
fn mathra_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(tex_to_svg, m)?)?;
    m.add_function(wrap_pyfunction!(tex_to_tree, m)?)?;
    m.add("__version__", mathra::VERSION)?;
    Ok(())
}
