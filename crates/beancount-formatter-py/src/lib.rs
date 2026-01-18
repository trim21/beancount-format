use beancount_formatter::configuration::{Configuration, NewLineKind};
use beancount_formatter::format;
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;

#[pyfunction(name = "format_text")]
#[pyo3(signature = (
  text,
  *,
  path = None,
  line_width = None,
  indent_width = None,
  new_line = None
))]
fn format_text_py(
  text: &str,
  path: Option<&str>,
  line_width: Option<u32>,
  indent_width: Option<u8>,
  new_line: Option<&str>,
) -> PyResult<String> {
  let mut config = Configuration::default();

  if let Some(value) = line_width {
    config.line_width = value;
  }

  if let Some(value) = indent_width {
    config.indent_width = value;
  }

  if let Some(value) = new_line {
    config.new_line = NewLineKind::parse(value).map_err(PyValueError::new_err)?;
  }

  let formatted = format(path, text, &config).map_err(|err| PyRuntimeError::new_err(err.to_string()))?;
  Ok(formatted)
}

#[pyfunction(name = "main")]
fn main_py(args: Vec<String>) -> PyResult<bool> {
  let outcome =
    beancount_formatter_cli::main_with_args(args).map_err(|err| PyRuntimeError::new_err(err.to_string()))?;

  Ok(outcome.any_changed)
}

#[pymodule]
fn beancount_format(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
  m.add_function(wrap_pyfunction!(format_text_py, m)?)?;
  m.add_function(wrap_pyfunction!(main_py, m)?)?;
  Ok(())
}
