use std::collections::HashMap;

/// Generic configuration value accepted by the formatter.
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigKeyValue {
  Bool(bool),
  Number(i64),
  Text(String),
}

impl From<bool> for ConfigKeyValue {
  fn from(value: bool) -> Self {
    ConfigKeyValue::Bool(value)
  }
}

impl From<i32> for ConfigKeyValue {
  fn from(value: i32) -> Self {
    ConfigKeyValue::Number(value as i64)
  }
}

impl From<u32> for ConfigKeyValue {
  fn from(value: u32) -> Self {
    ConfigKeyValue::Number(value as i64)
  }
}

impl From<u8> for ConfigKeyValue {
  fn from(value: u8) -> Self {
    ConfigKeyValue::Number(value as i64)
  }
}

impl From<&str> for ConfigKeyValue {
  fn from(value: &str) -> Self {
    ConfigKeyValue::Text(value.to_string())
  }
}

impl From<String> for ConfigKeyValue {
  fn from(value: String) -> Self {
    ConfigKeyValue::Text(value)
  }
}

pub type ConfigKeyMap = HashMap<String, ConfigKeyValue>;

#[derive(Debug, Clone, PartialEq)]
pub struct ConfigurationDiagnostic {
  pub property: String,
  pub message: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolveConfigurationResult<T> {
  pub config: T,
  pub diagnostics: Vec<ConfigurationDiagnostic>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalConfiguration {
  pub line_width: Option<u32>,
  pub use_tabs: Option<bool>,
  pub indent_width: Option<u8>,
  pub new_line_kind: Option<String>,
}

pub const DEFAULT_LINE_WIDTH: u32 = 120;
pub const DEFAULT_INDENT_WIDTH: u8 = 2;
pub const DEFAULT_NEW_LINE_KIND: &str = "lf";
pub const DEFAULT_USE_TABS: bool = false;

impl Default for GlobalConfiguration {
  fn default() -> Self {
    GlobalConfiguration {
      line_width: Some(DEFAULT_LINE_WIDTH),
      use_tabs: Some(DEFAULT_USE_TABS),
      indent_width: Some(DEFAULT_INDENT_WIDTH),
      new_line_kind: Some(DEFAULT_NEW_LINE_KIND.to_string()),
    }
  }
}
