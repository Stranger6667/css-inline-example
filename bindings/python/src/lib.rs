use pyo3::{create_exception, exceptions::ValueError, prelude::*, wrap_pyfunction};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const INLINE_ERROR_DOCSTRING: &str = "An error that can occur during CSS inlining";

create_exception!(css_inline, InlineError, ValueError);

struct InlineErrorWrapper(css_inline::InlineError);

impl From<InlineErrorWrapper> for PyErr {
    fn from(error: InlineErrorWrapper) -> Self {
        let message = match error.0 {
            css_inline::InlineError::IO(inner) => inner.to_string(),
            css_inline::InlineError::ParseError(message) => message,
        };
        InlineError::py_err(message)
    }
}

/// inline(html: str, remove_style_tags: bool = False) -> str
///
/// Inline CSS!
#[pyfunction]
fn inline(html: &str, remove_style_tags: Option<bool>) -> PyResult<String> {
    let options = css_inline::InlineOptions {
        remove_style_tags: remove_style_tags.unwrap_or(false),
    };
    let inliner = css_inline::CSSInliner::new(options);
    Ok(inliner.inline(html).map_err(InlineErrorWrapper)?)
}

/// CSSInliner(remove_style_tags=False)
///
/// Customizable CSS inliner.
#[pyclass(module = "css_inline")]
#[text_signature = "(remove_style_tags=False)"]
struct CSSInliner {
    inner: css_inline::CSSInliner,
}

#[pymethods]
impl CSSInliner {
    #[new]
    fn new(remove_style_tags: Option<bool>) -> Self {
        let options = css_inline::InlineOptions {
            remove_style_tags: remove_style_tags.unwrap_or(false),
        };
        Self {
            inner: css_inline::CSSInliner::new(options),
        }
    }

    /// inline(html)
    ///
    /// Inline CSS in the given HTML document
    #[text_signature = "(html)"]
    fn inline(&self, html: &str) -> PyResult<String> {
        Ok(self.inner.inline(html).map_err(InlineErrorWrapper)?)
    }
}

#[allow(dead_code)]
mod build {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

/// This docstring will be used for the Python module
/// and will be available as `css_inline.__doc__`
#[pymodule]
fn css_inline(py: Python, module: &PyModule) -> PyResult<()> {
    module.add_class::<CSSInliner>()?;
    module.add_wrapped(wrap_pyfunction!(inline))?;
    let inline_error = py.get_type::<InlineError>();
    inline_error.setattr("__doc__", INLINE_ERROR_DOCSTRING)?;
    module.add("InlineError", inline_error)?;
    module.add("__build__", pyo3_built::pyo3_built!(py, build))?;
    module.add("__version__", VERSION)?;
    Ok(())
}
