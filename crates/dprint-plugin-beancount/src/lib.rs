#![cfg(all(feature = "wasm", target_arch = "wasm32", target_os = "unknown"))]

use beancount_formatter::configuration::Configuration;
use beancount_formatter::configuration::NewLineKind;
use beancount_formatter::format_text;
use dprint_core::configuration::ConfigKeyMap;
use dprint_core::configuration::GlobalConfiguration;
use dprint_core::generate_plugin_code;
use dprint_core::plugins::CheckConfigUpdatesMessage;
use dprint_core::plugins::ConfigChange;
use dprint_core::plugins::FileMatchingInfo;
use dprint_core::plugins::FormatResult;
use dprint_core::plugins::PluginInfo;
use dprint_core::plugins::PluginResolveConfigurationResult;
use dprint_core::plugins::SyncFormatRequest;
use dprint_core::plugins::SyncHostFormatRequest;
use dprint_core::plugins::SyncPluginHandler;
use dprint_core::configuration::NewLineKind as DprintNewLineKind;

struct BeancountPluginHandler;

impl SyncPluginHandler<Configuration> for BeancountPluginHandler {
  fn resolve_config(
    &mut self,
    config: ConfigKeyMap,
    global_config: &GlobalConfiguration,
  ) -> PluginResolveConfigurationResult<Configuration> {
    let config = resolve_config_dprint(config, global_config);
    PluginResolveConfigurationResult {
      config: config.config,
      diagnostics: config.diagnostics,
      file_matching: FileMatchingInfo {
        file_extensions: vec!["beancount".to_string()],
        file_names: vec![],
      },
    }
  }

  fn check_config_updates(&self, _message: CheckConfigUpdatesMessage) -> Result<Vec<ConfigChange>, anyhow::Error> {
    Ok(Vec::new())
  }

  fn plugin_info(&mut self) -> PluginInfo {
    let version = env!("CARGO_PKG_VERSION").to_string();
    PluginInfo {
      name: env!("CARGO_PKG_NAME").to_string(),
      version: version.clone(),
      config_key: "beancount".to_string(),
      help_url: "https://github.com/dprint/dprint-plugin-beancount".to_string(),
      config_schema_url: format!(
        "https://plugins.dprint.dev/dprint/dprint-plugin-beancount/{}/schema.json",
        version
      ),
      update_url: Some("https://plugins.dprint.dev/dprint/dprint-plugin-beancount/latest.json".to_string()),
    }
  }

  fn license_text(&mut self) -> String {
    std::str::from_utf8(include_bytes!("../../../LICENSE")).unwrap().into()
  }

  fn format(
    &mut self,
    request: SyncFormatRequest<Configuration>,
    _format_with_host: impl FnMut(SyncHostFormatRequest) -> FormatResult,
  ) -> FormatResult {
    let file_text = String::from_utf8(request.file_bytes)?;
    format_text(request.file_path, &file_text, request.config).map(|maybe_text| maybe_text.map(|t| t.into_bytes()))
  }
}

generate_plugin_code!(BeancountPluginHandler, BeancountPluginHandler);

fn resolve_config_dprint(
  config: ConfigKeyMap,
  global_config: &GlobalConfiguration,
) -> dprint_core::configuration::ResolveConfigurationResult<Configuration> {
  use dprint_core::configuration::RECOMMENDED_GLOBAL_CONFIGURATION;
  use dprint_core::configuration::ResolveConfigurationResult;
  use dprint_core::configuration::get_unknown_property_diagnostics;
  use dprint_core::configuration::get_value;

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
    new_line_kind: map_new_line_kind(get_value(
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

fn map_new_line_kind(value: DprintNewLineKind) -> NewLineKind {
  match value {
    DprintNewLineKind::LineFeed => NewLineKind::LF,
    DprintNewLineKind::CarriageReturnLineFeed => NewLineKind::CRLF,
    _ => NewLineKind::LF,
  }
}
