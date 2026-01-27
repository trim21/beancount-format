#![cfg(all(target_arch = "wasm32", target_os = "unknown"))]

use beancount_formatter::configuration::Configuration;
use beancount_formatter::configuration::NewLineKind;
use beancount_formatter::format as format_beancount;
use dprint_core::configuration::ConfigKeyMap;
use dprint_core::configuration::GlobalConfiguration;
use dprint_core::configuration::NewLineKind as DprintNewLineKind;
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
        file_extensions: vec!["beancount".to_string(), "bean".to_string()],
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
      name: "@trim21/dprint-plugin-beancount".to_string(),
      version: version.clone(),
      config_key: "beancount".to_string(),
      help_url: "https://github.com/trim21/beancount-format".to_string(),
      config_schema_url: format!(
        "https://cdn.jsdelivr.net/gh/trim21/beancount-format@gh-pages/dprint-plugin-beancount/{}/schema.json",
        version
      ),
      update_url: Some(
        "https://cdn.jsdelivr.net/gh/trim21/beancount-format@gh-pages/dprint-plugin-beancount/latest.json".to_string(),
      ),
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
    if file_text.trim().is_empty() {
      return Ok(Some(Vec::new()));
    }
    let formatted = format_beancount(request.file_path.to_str(), &file_text, &request.config)?;

    if formatted == file_text {
      Ok(None)
    } else {
      Ok(Some(formatted.into_bytes()))
    }
  }
}

generate_plugin_code!(BeancountPluginHandler, BeancountPluginHandler);

fn resolve_config_dprint(
  config: ConfigKeyMap,
  global_config: &GlobalConfiguration,
) -> dprint_core::configuration::ResolveConfigurationResult<Configuration> {
  use dprint_core::configuration::ResolveConfigurationResult;
  use dprint_core::configuration::get_unknown_property_diagnostics;
  use dprint_core::configuration::get_value;

  let default = Configuration::default();

  let mut diagnostics = Vec::new();
  let mut config = config;

  let resolved_config = Configuration {
    line_width: get_value(
      &mut config,
      "line_width",
      global_config.line_width.unwrap_or(default.line_width),
      &mut diagnostics,
    ),
    indent_width: get_value(
      &mut config,
      "indent_width",
      global_config.indent_width.unwrap_or(default.indent_width),
      &mut diagnostics,
    ),
    new_line: map_new_line_kind(get_value(
      &mut config,
      "new_line",
      global_config.new_line_kind.unwrap_or(match default.new_line {
        NewLineKind::LF => DprintNewLineKind::LineFeed,
        NewLineKind::CRLF => DprintNewLineKind::CarriageReturnLineFeed,
      }),
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
