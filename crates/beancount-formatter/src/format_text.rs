use std::path::Path;

use anyhow::Result;

use crate::configuration::Configuration;
use crate::format::format;

/// Formats the provided beancount text and returns `Ok(Some(String))` when the
/// formatter changed the text or `Ok(None)` when no edits were necessary.
pub fn format_text(_path: &Path, text: &str, config: &Configuration) -> Result<Option<String>> {
  let result = format(text, config)?;
  if result == text { Ok(None) } else { Ok(Some(result)) }
}
