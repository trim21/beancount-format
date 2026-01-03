use super::*;

/// Formatting configuration builder.
///
/// # Example
///
/// ```
/// use beancount_formatter::configuration::*;
///
/// let config = ConfigurationBuilder::new()
///     .line_width(80)
///     .build();
/// ```
#[derive(Default)]
pub struct ConfigurationBuilder {
  pub(super) config: ConfigKeyMap,
  global_config: Option<GlobalConfiguration>,
}

impl ConfigurationBuilder {
  /// Constructs a new configuration builder.
  pub fn new() -> ConfigurationBuilder {
    Default::default()
  }

  /// Gets the final configuration that can be used to format a file.
  pub fn build(&self) -> Configuration {
    if let Some(global_config) = &self.global_config {
      resolve_config(self.config.clone(), global_config).config
    } else {
      let config = self.config.clone();
      resolve_config(config, &GlobalConfiguration::default()).config
    }
  }

  /// Set the global configuration.
  pub fn global_config(&mut self, global_config: GlobalConfiguration) -> &mut Self {
    self.global_config = Some(global_config);
    self
  }

  /// The width of a line the the printer will try to stay under. Note that the printer may exceed this width in certain cases.
  /// Default: 120
  pub fn line_width(&mut self, value: u32) -> &mut Self {
    self.insert("line_width", (value as i32).into())
  }

  /// The number of columns for an indent.
  ///
  /// Default: `2`
  pub fn indent_width(&mut self, value: u8) -> &mut Self {
    self.insert("indent_width", (value as i32).into())
  }

  /// The kind of newline to use.
  /// Default: `NewLineKind::LF`
  pub fn new_line_kind(&mut self, value: NewLineKind) -> &mut Self {
    self.insert("new_line_kind", value.as_str().into())
  }

  /// Use this prefix width instead of determining an optimal value automatically.
  pub fn prefix_width(&mut self, value: usize) -> &mut Self {
    self.insert("prefix_width", value.into())
  }

  /// Use this width to render numbers instead of determining an optimal value.
  pub fn num_width(&mut self, value: usize) -> &mut Self {
    self.insert("num_width", value.into())
  }

  /// Align currencies in this column.
  pub fn currency_column(&mut self, value: usize) -> &mut Self {
    self.insert("currency_column", value.into())
  }

  /// Spacing between account names and amounts.
  pub fn account_amount_spacing(&mut self, value: usize) -> &mut Self {
    self.insert("account_amount_spacing", value.into())
  }

  /// Number of spaces between the number and currency.
  pub fn number_currency_spacing(&mut self, value: usize) -> &mut Self {
    self.insert("number_currency_spacing", value.into())
  }

  #[cfg(test)]
  pub(super) fn get_inner_config(&self) -> ConfigKeyMap {
    self.config.clone()
  }

  fn insert(&mut self, name: &str, value: ConfigKeyValue) -> &mut Self {
    self.config.insert(String::from(name), value);
    self
  }
}

#[cfg(test)]
mod tests {
  use super::NewLineKind;
  use super::*;

  #[test]
  fn check_all_values_set() {
    let mut config = ConfigurationBuilder::new();
    config
      .new_line_kind(NewLineKind::CRLF)
      .line_width(90)
      .indent_width(4)
      .new_line_kind(NewLineKind::CRLF)
      .prefix_width(12)
      .num_width(8)
      .currency_column(40)
      .account_amount_spacing(2)
      .number_currency_spacing(3);

    let inner_config = config.get_inner_config();
    assert_eq!(inner_config.len(), 8);
    let diagnostics = resolve_config(inner_config, &GlobalConfiguration::default()).diagnostics;
    assert_eq!(diagnostics.len(), 0);
  }

  #[test]
  fn handle_global_config() {
    let mut global_config = ConfigKeyMap::new();
    global_config.insert(String::from("line_width"), 90.into());
    global_config.insert(String::from("new_line_kind"), "crlf".into());
    global_config.insert(String::from("use_tabs"), true.into());
    let global_config = resolve_global_config(&mut global_config).config;
    let mut config_builder = ConfigurationBuilder::new();
    let config = config_builder.global_config(global_config).build();
    assert_eq!(config.line_width, 90);
    assert!(config.new_line_kind == NewLineKind::CRLF);
  }

  #[test]
  fn use_defaults_when_global_not_set() {
    let global_config = GlobalConfiguration::default();
    let mut config_builder = ConfigurationBuilder::new();
    let config = config_builder.global_config(global_config).build();
    assert_eq!(config.indent_width, 2);
    assert!(config.new_line_kind == NewLineKind::LF);
  }
}
