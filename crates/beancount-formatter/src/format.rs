use anyhow::Result;

use crate::configuration::Configuration;

/// Formats beancount source text according to the provided configuration.
///
/// Currently this is a pass-through implementation; it will be fleshed out later.
pub fn format(source_text: &str, _config: &Configuration) -> Result<String> {
  Ok(source_text.to_string())
}
