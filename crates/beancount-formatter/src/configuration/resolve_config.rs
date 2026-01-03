use super::{Configuration, NewLineKind};
use dprint_core::configuration::*;

/// Resolves configuration from a collection of key value strings.
///
/// # Example
///
/// ```
/// use dprint_core::configuration::ConfigKeyMap;
/// use dprint_core::configuration::resolve_global_config;
/// use beancount_formatter::configuration::resolve_config;
///
/// let mut config_map = ConfigKeyMap::new();
/// let global_config_result = resolve_global_config(&mut config_map);
///
/// // check global_config_result.diagnostics here...
///
/// let beancount_config_map = ConfigKeyMap::new();
/// let config_result = resolve_config(
///     beancount_config_map,
///     &global_config_result.config
/// );
///
/// // check config_result.diagnostics here and use config_result.config
/// ```
pub fn resolve_config(
  config: ConfigKeyMap,
  global_config: &GlobalConfiguration,
) -> ResolveConfigurationResult<Configuration> {
  let mut diagnostics = Vec::new();
  let mut config = config;

  let resolved_config = Configuration {
    line_width: get_value(
      &mut config,
      "lineWidth",
      global_config
        .line_width
        .unwrap_or(RECOMMENDED_GLOBAL_CONFIGURATION.line_width),
      &mut diagnostics,
    ),
    use_tabs: get_value(
      &mut config,
      "useTabs",
      global_config
        .use_tabs
        .unwrap_or(RECOMMENDED_GLOBAL_CONFIGURATION.use_tabs),
      &mut diagnostics,
    ),
    indent_width: get_value(
      &mut config,
      "indentWidth",
      global_config.indent_width.unwrap_or(2),
      &mut diagnostics,
    ),
    new_line_kind: NewLineKind::from(get_value(
      &mut config,
      "newLineKind",
      global_config
        .new_line_kind
        .unwrap_or(RECOMMENDED_GLOBAL_CONFIGURATION.new_line_kind),
      &mut diagnostics,
    )),
  };

  diagnostics.extend(get_unknown_property_diagnostics(config));

  ResolveConfigurationResult {
    config: resolved_config,
    diagnostics,
  }
}
