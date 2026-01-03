use std::path::PathBuf;

use beancount_formatter::configuration::ConfigKeyMap;
use beancount_formatter::configuration::GlobalConfiguration;
use beancount_formatter::configuration::resolve_config;
use beancount_formatter::format_text;

#[test]
fn formats_without_changes_returns_none() {
  let global_config = GlobalConfiguration::default();
  let config_result = resolve_config(ConfigKeyMap::new(), &global_config);
  assert!(config_result.diagnostics.is_empty());

  let result = format_text(
    &PathBuf::from("example.beancount"),
    "2010-01-01 open Assets:Cash\n",
    &config_result.config,
  )
  .unwrap();

  assert!(result.is_none());
}
