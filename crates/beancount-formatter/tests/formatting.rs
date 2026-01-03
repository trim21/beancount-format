use std::path::PathBuf;

use beancount_formatter::configuration::resolve_config;
use beancount_formatter::format_text;
use dprint_core::configuration::*;
use dprint_development::ensure_no_diagnostics;

#[test]
fn formats_without_changes_returns_none() {
  let global_config = GlobalConfiguration::default();
  let config_result = resolve_config(ConfigKeyMap::new(), &global_config);
  ensure_no_diagnostics(&config_result.diagnostics);

  let result = format_text(
    &PathBuf::from("example.beancount"),
    "2010-01-01 open Assets:Cash\n",
    &config_result.config,
  )
  .unwrap();

  assert!(result.is_none());
}
