use super::config_types::ConfigKeyMap;
use super::config_types::ConfigKeyValue;
use super::config_types::ConfigurationDiagnostic;
use super::config_types::DEFAULT_INDENT_WIDTH;
use super::config_types::DEFAULT_LINE_WIDTH;
use super::config_types::DEFAULT_NEW_LINE_KIND;
use super::config_types::GlobalConfiguration;
use super::config_types::ResolveConfigurationResult;
use super::{Configuration, NewLineKind};

/// Resolves configuration from a collection of key value strings.
///
/// # Example
///
/// ```
/// use beancount_formatter::configuration::resolve_config;
/// use beancount_formatter::configuration::resolve_global_config;
/// use beancount_formatter::configuration::ConfigKeyMap;
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

  let line_width_default = global_config.line_width.unwrap_or(DEFAULT_LINE_WIDTH);
  let indent_width_default = global_config.indent_width.unwrap_or(DEFAULT_INDENT_WIDTH);
  let new_line_kind_default = global_config
    .new_line_kind
    .as_deref()
    .unwrap_or(DEFAULT_NEW_LINE_KIND)
    .to_string();

  let format_indent_override =
    get_usize_option_keys(&mut config, &["format_indent_width"], &mut diagnostics);

  let indent_width_value = match format_indent_override {
    Some(value) => match u8::try_from(value) {
      Ok(v) => v,
      Err(_) => {
        diagnostics.push(ConfigurationDiagnostic {
          property: "format_indent_width".to_string(),
          message: "Expected a value between 0 and 255 for 'format_indent_width'.".to_string(),
        });
        indent_width_default
      }
    },
    None => get_u8(&mut config, "indent_width", indent_width_default, &mut diagnostics),
  };

  let resolved_config = Configuration {
    line_width: get_u32(&mut config, "line_width", line_width_default, &mut diagnostics),
    indent_width: indent_width_value,
    new_line_kind: parse_new_line_kind(
      &mut config,
      "new_line_kind",
      &new_line_kind_default,
      &mut diagnostics,
    ),
    prefix_width: get_usize_option_keys(&mut config, &["prefix_width"], &mut diagnostics),
    num_width: get_usize_option_keys(&mut config, &["num_width"], &mut diagnostics),
    currency_column: get_usize_option_keys(&mut config, &["currency_column"], &mut diagnostics),
    account_amount_spacing: get_usize_option_keys(&mut config, &["account_amount_spacing"], &mut diagnostics),
    number_currency_spacing: get_usize_option_keys(&mut config, &["number_currency_spacing"], &mut diagnostics),
  };

  diagnostics.extend(get_unknown_property_diagnostics(config));

  ResolveConfigurationResult {
    config: resolved_config,
    diagnostics,
  }
}

pub fn resolve_global_config(config: &mut ConfigKeyMap) -> ResolveConfigurationResult<GlobalConfiguration> {
  let mut diagnostics = Vec::new();
  let mut config = std::mem::take(config);

  let resolved_config = GlobalConfiguration {
    line_width: Some(get_u32(&mut config, "line_width", DEFAULT_LINE_WIDTH, &mut diagnostics)),
    indent_width: Some(get_u8(
      &mut config,
      "indent_width",
      DEFAULT_INDENT_WIDTH,
      &mut diagnostics,
    )),
    new_line_kind: Some(get_string(&mut config, "new_line_kind", DEFAULT_NEW_LINE_KIND)),
  };

  diagnostics.extend(get_unknown_property_diagnostics(config));

  ResolveConfigurationResult {
    config: resolved_config,
    diagnostics,
  }
}

fn get_unknown_property_diagnostics(config: ConfigKeyMap) -> Vec<ConfigurationDiagnostic> {
  config
    .into_keys()
    .map(|key| ConfigurationDiagnostic {
      property: key,
      message: "Unknown property in configuration.".to_string(),
    })
    .collect()
}

fn get_u32(
  config: &mut ConfigKeyMap,
  name: &str,
  default_value: u32,
  diagnostics: &mut Vec<ConfigurationDiagnostic>,
) -> u32 {
  match config.remove(name) {
    Some(ConfigKeyValue::Number(value)) if value >= 0 => value as u32,
    Some(ConfigKeyValue::Text(text)) => match text.parse::<u32>() {
      Ok(value) => value,
      Err(_) => {
        diagnostics.push(ConfigurationDiagnostic {
          property: name.to_string(),
          message: format!("Expected a positive number for '{}'.", name),
        });
        default_value
      }
    },
    Some(_) => {
      diagnostics.push(ConfigurationDiagnostic {
        property: name.to_string(),
        message: format!("Expected a positive number for '{}'.", name),
      });
      default_value
    }
    None => default_value,
  }
}

fn get_u8(
  config: &mut ConfigKeyMap,
  name: &str,
  default_value: u8,
  diagnostics: &mut Vec<ConfigurationDiagnostic>,
) -> u8 {
  let value = get_u32(config, name, default_value as u32, diagnostics);
  value as u8
}

fn get_usize_option_keys(
  config: &mut ConfigKeyMap,
  names: &[&str],
  diagnostics: &mut Vec<ConfigurationDiagnostic>,
) -> Option<usize> {
  for name in names {
    match config.remove(*name) {
      Some(ConfigKeyValue::Number(value)) if value >= 0 => return Some(value as usize),
      Some(ConfigKeyValue::Text(text)) => match text.parse::<usize>() {
        Ok(value) => return Some(value),
        Err(_) => {
          diagnostics.push(ConfigurationDiagnostic {
            property: (*name).to_string(),
            message: format!("Expected a positive number for '{}'.", name),
          });
          return None;
        }
      },
      Some(_) => {
        diagnostics.push(ConfigurationDiagnostic {
          property: (*name).to_string(),
          message: format!("Expected a positive number for '{}'.", name),
        });
        return None;
      }
      None => continue,
    }
  }
  None
}

fn get_string(config: &mut ConfigKeyMap, name: &str, default_value: &str) -> String {
  match config.remove(name) {
    Some(ConfigKeyValue::Text(value)) => value,
    Some(ConfigKeyValue::Bool(value)) => value.to_string(),
    Some(ConfigKeyValue::Number(value)) => value.to_string(),
    None => default_value.to_string(),
  }
}

fn parse_new_line_kind(
  config: &mut ConfigKeyMap,
  name: &str,
  default_value: &str,
  diagnostics: &mut Vec<ConfigurationDiagnostic>,
) -> NewLineKind {
  let raw_value = get_string(config, name, default_value);
  match NewLineKind::parse(&raw_value) {
    Ok(kind) => kind,
    Err(_) => {
      diagnostics.push(ConfigurationDiagnostic {
        property: name.to_string(),
        message: format!("Unsupported new_line_kind: {}", raw_value),
      });
      NewLineKind::parse(default_value).unwrap_or(NewLineKind::LF)
    }
  }
}
