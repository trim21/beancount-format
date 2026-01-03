#![cfg(feature = "python")]

use std::path::Path;

use beancount_formatter::configuration::{ConfigurationBuilder, NewLineKind};
use beancount_formatter::format_text;
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;

#[pyfunction(name = "format_text")]
#[pyo3(signature = (
  text,
  *,
  line_width = None,
  use_tabs = None,
  indent_width = None,
  new_line_kind = None
))]
fn format_text_py(
  text: &str,
  line_width: Option<u32>,
  use_tabs: Option<bool>,
  indent_width: Option<u8>,
  new_line_kind: Option<&str>,
) -> PyResult<String> {
  let mut config_builder = ConfigurationBuilder::new();

  if let Some(value) = line_width {
    config_builder.line_width(value);
  }

  if let Some(value) = use_tabs {
    config_builder.use_tabs(value);
  }

  if let Some(value) = indent_width {
    config_builder.indent_width(value);
  }

  if let Some(value) = new_line_kind {
    let parsed = NewLineKind::parse(value).map_err(PyValueError::new_err)?;
    config_builder.new_line_kind(parsed);
  }

  let config = config_builder.build();
  let result = format_text(Path::new("example.beancount"), text, &config)
    .map_err(|err| PyRuntimeError::new_err(err.to_string()))?;
  Ok(result.unwrap_or_else(|| text.to_string()))
}

#[pymodule]
fn bean_format(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
  m.add_function(wrap_pyfunction!(format_text_py, m)?)?;
  Ok(())
}
