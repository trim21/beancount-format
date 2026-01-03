use beancount_formatter::configuration::resolve_config;
use beancount_formatter::configuration::resolve_global_config;
use beancount_formatter::configuration::ConfigKeyMap;
use beancount_formatter::format;

#[test]
fn formats_without_changes_returns_none() {
  let mut global_config = ConfigKeyMap::new();
  let global_result = resolve_global_config(&mut global_config);
  assert!(global_result.diagnostics.is_empty());

  let config_result = resolve_config(ConfigKeyMap::new(), &global_result.config);
  assert!(config_result.diagnostics.is_empty());

  let result = format(
    Some("example.beancount"),
    "2010-01-01 open Assets:Cash\n",
    &config_result.config,
  )
  .unwrap();

  assert_eq!(result, "2010-01-01 open Assets:Cash\n");
}
